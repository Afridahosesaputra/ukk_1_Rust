use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{Local, NaiveDate};
use crate::auth::{extract_auth_user, require_role};
use crate::db::Db;
use crate::models::{ApiResponse, Pengembalian, PengembalianRequest, Role};

pub async fn list_pengembalian(
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<Vec<Pengembalian>>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    let state = db.read().await;
    let mut returns = state.pengembalian.clone();

    // Jika peminjam biasa, hanya lihat riwayat pengembaliannya sendiri
    if auth_user.role == Role::Peminjam {
        returns.retain(|r| r.user_id == auth_user.id);
    }

    returns.reverse();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Berhasil memuat {} data pengembalian.", returns.len()),
            data: Some(returns),
        }),
    )
}

pub async fn proses_pengembalian(
    headers: HeaderMap,
    State(db): State<Db>,
    Json(payload): Json<PengembalianRequest>,
) -> (StatusCode, Json<ApiResponse<Pengembalian>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;

    // Cari peminjaman yang berstatus dipinjam
    let pinjam_idx = match state.peminjaman.iter().position(|p| p.id == payload.peminjaman_id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Data peminjaman tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    if state.peminjaman[pinjam_idx].status != "dipinjam" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                message: format!(
                    "Hanya peminjaman dengan status 'dipinjam' yang dapat dikembalikan! Status saat ini: '{}'",
                    state.peminjaman[pinjam_idx].status
                ),
                data: None,
            }),
        );
    }

    let tgl_kembali_str = payload.tanggal_kembali.trim();
    let tgl_rencana_str = &state.peminjaman[pinjam_idx].tanggal_kembali_rencana;

    // Kalkulasi hari terlambat
    let tgl_kembali_parsed = NaiveDate::parse_from_str(tgl_kembali_str, "%Y-%m-%d");
    let tgl_rencana_parsed = NaiveDate::parse_from_str(tgl_rencana_str, "%Y-%m-%d");

    let hari_terlambat = match (tgl_kembali_parsed, tgl_rencana_parsed) {
        (Ok(tgl_k), Ok(tgl_r)) => {
            let diff = tgl_k.signed_duration_since(tgl_r).num_days();
            if diff > 0 { diff as i32 } else { 0 }
        }
        _ => 0,
    };

    // Ambil tarif denda alat
    let alat_id = state.peminjaman[pinjam_idx].alat_id;
    let tarif_denda_harian = state.alat.iter()
        .find(|a| a.id == alat_id)
        .map(|a| a.tarif_denda_harian)
        .unwrap_or(5000.0);

    let denda_keterlambatan = (hari_terlambat as f64) * tarif_denda_harian;

    // Kalkulasi denda kerusakan fisik
    let kondisi_kembali = payload.kondisi_kembali.trim().to_lowercase();
    let denda_kerusakan = if let Some(manual_denda) = payload.denda_kerusakan {
        manual_denda.max(0.0)
    } else {
        match kondisi_kembali.as_str() {
            "rusak_ringan" => 25000.0,
            "rusak_berat" => 100000.0,
            _ => 0.0,
        }
    };

    let total_denda = denda_keterlambatan + denda_kerusakan;
    let status_pembayaran = if total_denda > 0.0 {
        "belum_lunas".to_string()
    } else {
        "tidak_ada_denda".to_string()
    };

    let new_kembali_id = state.next_pengembalian_id();
    let now = Local::now();
    let kode_kembali = format!("KMB-{}-{:03}", now.format("%Y%m%d"), new_kembali_id);
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let pinjam = &mut state.peminjaman[pinjam_idx];
    pinjam.status = "selesai".to_string();
    pinjam.tanggal_kembali_aktual = Some(tgl_kembali_str.to_string());
    pinjam.total_denda = total_denda;

    let pinjam_id = pinjam.id;
    let kode_pinjam = pinjam.kode_pinjam.clone();
    let user_id = pinjam.user_id;
    let nama_peminjam = pinjam.peminjam_nama.clone();
    let nama_alat = pinjam.nama_alat.clone();
    let jumlah_kembali = pinjam.jumlah;

    // Pulihkan stok alat
    if let Some(alat_item) = state.alat.iter_mut().find(|a| a.id == alat_id) {
        alat_item.stok_tersedia = (alat_item.stok_tersedia + jumlah_kembali).min(alat_item.stok_total);
    }

    // Update detail peminjaman
    if let Some(detail) = state.detail_peminjaman.iter_mut().find(|d| d.peminjaman_id == pinjam_id) {
        detail.kondisi_saat_kembali = kondisi_kembali;
        detail.denda_kerusakan = denda_kerusakan;
    }

    let pengembalian = Pengembalian {
        id: new_kembali_id,
        kode_kembali: kode_kembali.clone(),
        peminjaman_id: pinjam_id,
        kode_pinjam: kode_pinjam.clone(),
        user_id,
        nama_peminjam,
        petugas_id: auth_user.id,
        nama_petugas: auth_user.nama_lengkap.clone(),
        nama_alat: nama_alat.clone(),
        jumlah: jumlah_kembali,
        tanggal_kembali: tgl_kembali_str.to_string(),
        hari_terlambat,
        denda_keterlambatan,
        denda_kerusakan,
        total_denda,
        status_pembayaran,
        catatan: payload.catatan.unwrap_or_else(|| "Pengembalian berhasil diproses.".to_string()),
        created_at: now_str,
    };

    state.pengembalian.push(pengembalian.clone());

    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "PENGEMBALIAN",
        "PENGEMBALIAN",
        &format!(
            "Memproses pengembalian {} ({}) oleh {}. Telat: {} hari, Total Denda: Rp {:.0}",
            kode_kembali, kode_pinjam, auth_user.username, hari_terlambat, total_denda
        ),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!(
                "Pengembalian {} berhasil diproses! Stok alat telah dipulihkan. Total denda: Rp {:.0}",
                kode_kembali, total_denda
            ),
            data: Some(pengembalian),
        }),
    )
}

pub async fn bayar_denda(
    Path(id): Path<i64>,
    headers: HeaderMap,
    State(db): State<Db>,
) -> (StatusCode, Json<ApiResponse<Pengembalian>>) {
    let auth_user = match extract_auth_user(&headers, &db).await {
        Ok(u) => u,
        Err((status, err)) => return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None })),
    };

    if let Err((status, err)) = require_role(&auth_user, &[Role::Admin, Role::Petugas]) {
        return (status, Json(ApiResponse { success: false, message: err.message.clone(), data: None }));
    }

    let mut state = db.write().await;
    let idx = match state.pengembalian.iter().position(|p| p.id == id) {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    message: "Data pengembalian tidak ditemukan.".to_string(),
                    data: None,
                }),
            );
        }
    };

    state.pengembalian[idx].status_pembayaran = "lunas".to_string();
    let updated = state.pengembalian[idx].clone();

    state.add_log(
        Some(auth_user.id),
        &auth_user.username,
        auth_user.role.as_str(),
        "BAYAR_DENDA",
        "PENGEMBALIAN",
        &format!("Pembayaran denda {} sebesar Rp {:.0} dinyatakan LUNAS", updated.kode_kembali, updated.total_denda),
    );
    state.save_to_disk();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Denda untuk {} berhasil diubah menjadi LUNAS.", updated.kode_kembali),
            data: Some(updated),
        }),
    )
}
