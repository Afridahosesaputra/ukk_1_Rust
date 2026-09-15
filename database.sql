-- =============================================================================
-- UJI KOMPETENSI KEAHLIAN (UKK) REKAYASA PERANGKAT LUNAK (RPL) 2025/2026
-- SOAL PRAKTIK KEJURUAN: PENGEMBANGAN APLIKASI PEMINJAMAN ALAT
-- KODE: KM25.4.1.1
-- BASIS DATA: db_peminjaman_alat
-- =============================================================================

-- 1. PEMBUATAN DATABASE
CREATE DATABASE IF NOT EXISTS db_peminjaman_alat;
USE db_peminjaman_alat;

-- Hapus tabel lama jika ada (urutan drop memperhatikan foreign key)
DROP TABLE IF EXISTS log_aktivitas;
DROP TABLE IF EXISTS pengembalian;
DROP TABLE IF EXISTS detail_peminjaman;
DROP TABLE IF EXISTS peminjaman;
DROP TABLE IF EXISTS alat;
DROP TABLE IF EXISTS kategori;
DROP TABLE IF EXISTS users;

-- =============================================================================
-- 2. STRUKTUR TABEL (DDL) DENGAN INTEGRITAS RELASIONAL
-- =============================================================================

-- TABEL 1: USERS (3 Level Pengguna: Admin, Petugas, Peminjam)
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    nama_lengkap VARCHAR(100) NOT NULL,
    no_telepon VARCHAR(20),
    alamat TEXT,
    role ENUM('admin', 'petugas', 'peminjam') NOT NULL DEFAULT 'peminjam',
    status ENUM('aktif', 'nonaktif') NOT NULL DEFAULT 'aktif',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- TABEL 2: KATEGORI ALAT
CREATE TABLE kategori (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nama_kategori VARCHAR(100) NOT NULL UNIQUE,
    deskripsi TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- TABEL 3: ALAT (Inventaris Alat)
CREATE TABLE alat (
    id INT AUTO_INCREMENT PRIMARY KEY,
    kode_alat VARCHAR(30) NOT NULL UNIQUE,
    nama_alat VARCHAR(150) NOT NULL,
    kategori_id INT NOT NULL,
    stok_total INT NOT NULL DEFAULT 1,
    stok_tersedia INT NOT NULL DEFAULT 1,
    kondisi ENUM('baik', 'rusak_ringan', 'rusak_berat') NOT NULL DEFAULT 'baik',
    lokasi VARCHAR(100) DEFAULT 'Gudang Utama',
    spesifikasi TEXT,
    tarif_denda_harian DECIMAL(10,2) NOT NULL DEFAULT 5000.00,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_alat_kategori FOREIGN KEY (kategori_id) 
        REFERENCES kategori(id) 
        ON UPDATE CASCADE 
        ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- TABEL 4: PEMINJAMAN (Header Transaksi Peminjaman)
CREATE TABLE peminjaman (
    id INT AUTO_INCREMENT PRIMARY KEY,
    kode_pinjam VARCHAR(30) NOT NULL UNIQUE,
    user_id INT NOT NULL,
    tanggal_pengajuan TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    tanggal_pinjam DATE NOT NULL,
    tanggal_kembali_rencana DATE NOT NULL,
    tanggal_kembali_aktual DATE NULL,
    status ENUM('menunggu', 'disetujui', 'ditolak', 'dipinjam', 'selesai', 'dibatalkan') NOT NULL DEFAULT 'menunggu',
    keperluan TEXT,
    disetujui_oleh INT NULL,
    catatan_petugas TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_peminjaman_user FOREIGN KEY (user_id) 
        REFERENCES users(id) 
        ON UPDATE CASCADE 
        ON DELETE RESTRICT,
    CONSTRAINT fk_peminjaman_petugas FOREIGN KEY (disetujui_oleh) 
        REFERENCES users(id) 
        ON UPDATE CASCADE 
        ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- TABEL 5: DETAIL PEMINJAMAN (Detail Alat yang Dipinjam)
CREATE TABLE detail_peminjaman (
    id INT AUTO_INCREMENT PRIMARY KEY,
    peminjaman_id INT NOT NULL,
    alat_id INT NOT NULL,
    jumlah INT NOT NULL DEFAULT 1,
    kondisi_saat_pinjam VARCHAR(50) DEFAULT 'baik',
    kondisi_saat_kembali VARCHAR(50) DEFAULT 'baik',
    denda_kerusakan DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    CONSTRAINT fk_detail_peminjaman FOREIGN KEY (peminjaman_id) 
        REFERENCES peminjaman(id) 
        ON UPDATE CASCADE 
        ON DELETE CASCADE,
    CONSTRAINT fk_detail_alat FOREIGN KEY (alat_id) 
        REFERENCES alat(id) 
        ON UPDATE CASCADE 
        ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- TABEL 6: PENGEMBALIAN & DENDA
CREATE TABLE pengembalian (
    id INT AUTO_INCREMENT PRIMARY KEY,
    kode_kembali VARCHAR(30) NOT NULL UNIQUE,
    peminjaman_id INT NOT NULL UNIQUE,
    user_id INT NOT NULL,
    petugas_id INT NOT NULL,
    tanggal_kembali DATE NOT NULL,
    hari_terlambat INT NOT NULL DEFAULT 0,
    denda_keterlambatan DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    denda_kerusakan DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    total_denda DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    status_pembayaran ENUM('lunas', 'belum_lunas', 'tidak_ada_denda') NOT NULL DEFAULT 'tidak_ada_denda',
    catatan TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_pengembalian_peminjaman FOREIGN KEY (peminjaman_id) 
        REFERENCES peminjaman(id) 
        ON UPDATE CASCADE 
        ON DELETE RESTRICT,
    CONSTRAINT fk_pengembalian_user FOREIGN KEY (user_id) 
        REFERENCES users(id) 
        ON UPDATE CASCADE 
        ON DELETE RESTRICT,
    CONSTRAINT fk_pengembalian_petugas FOREIGN KEY (petugas_id) 
        REFERENCES users(id) 
        ON UPDATE CASCADE 
        ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- TABEL 7: LOG AKTIVITAS (Audit Trail Sistem)
CREATE TABLE log_aktivitas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NULL,
    username VARCHAR(50),
    role VARCHAR(20),
    aksi VARCHAR(50) NOT NULL,
    modul VARCHAR(50) NOT NULL,
    detail TEXT NOT NULL,
    ip_address VARCHAR(45) DEFAULT '127.0.0.1',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_log_user FOREIGN KEY (user_id) 
        REFERENCES users(id) 
        ON UPDATE CASCADE 
        ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- =============================================================================
-- 3. FUNCTION: PERHITUNGAN DENDA KETERLAMBATAN & KERUSAKAN
-- =============================================================================
DELIMITER $$

CREATE FUNCTION fn_hitung_denda_keterlambatan(
    p_tgl_rencana DATE,
    p_tgl_aktual DATE,
    p_tarif_harian DECIMAL(10,2)
) RETURNS DECIMAL(10,2)
DETERMINISTIC
BEGIN
    DECLARE v_selisih_hari INT;
    DECLARE v_total_denda DECIMAL(10,2);

    SET v_selisih_hari = DATEDIFF(p_tgl_aktual, p_tgl_rencana);
    
    IF v_selisih_hari > 0 THEN
        SET v_total_denda = v_selisih_hari * p_tarif_harian;
    ELSE
        SET v_total_denda = 0.00;
    END IF;

    RETURN v_total_denda;
END$$

DELIMITER ;

-- =============================================================================
-- 4. STORED PROCEDURES (PROSEDUR TERORGANISASI)
-- =============================================================================
DELIMITER $$

-- Prosedur 1: Mengajukan Peminjaman dengan Validasi Stok
CREATE PROCEDURE sp_ajukan_peminjaman(
    IN p_kode_pinjam VARCHAR(30),
    IN p_user_id INT,
    IN p_alat_id INT,
    IN p_jumlah INT,
    IN p_tgl_pinjam DATE,
    IN p_tgl_kembali_rencana DATE,
    IN p_keperluan TEXT,
    OUT p_status_code INT,
    OUT p_message VARCHAR(255)
)
BEGIN
    DECLARE v_stok_tersedia INT;
    DECLARE v_pinjam_id INT;

    -- Handler jika terjadi exception/error SQL
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        SET p_status_code = 500;
        SET p_message = 'Terjadi kesalahan sistem saat memproses peminjaman.';
    END;

    START TRANSACTION;

    -- Periksa ketersediaan stok alat
    SELECT stok_tersedia INTO v_stok_tersedia 
    FROM alat 
    WHERE id = p_alat_id FOR UPDATE;

    IF v_stok_tersedia IS NULL THEN
        ROLLBACK;
        SET p_status_code = 404;
        SET p_message = 'Alat tidak ditemukan.';
    ELSEIF v_stok_tersedia < p_jumlah THEN
        ROLLBACK;
        SET p_status_code = 400;
        SET p_message = CONCAT('Stok tidak mencukupi. Stok tersedia: ', v_stok_tersedia);
    ELSE
        -- Masukkan ke header peminjaman
        INSERT INTO peminjaman (
            kode_pinjam, user_id, tanggal_pinjam, tanggal_kembali_rencana, 
            status, keperluan
        ) VALUES (
            p_kode_pinjam, p_user_id, p_tgl_pinjam, p_tgl_kembali_rencana, 
            'menunggu', p_keperluan
        );
        
        SET v_pinjam_id = LAST_INSERT_ID();

        -- Masukkan detail peminjaman
        INSERT INTO detail_peminjaman (
            peminjaman_id, alat_id, jumlah, kondisi_saat_pinjam
        ) VALUES (
            v_pinjam_id, p_alat_id, p_jumlah, 'baik'
        );

        -- Catat log aktivitas
        INSERT INTO log_aktivitas (user_id, username, role, aksi, modul, detail)
        VALUES (p_user_id, 'user', 'peminjam', 'PENGAJUAN', 'PEMINJAMAN', 
                CONCAT('Mengajukan pinjam kode: ', p_kode_pinjam, ', alat ID: ', p_alat_id, ', jumlah: ', p_jumlah));

        COMMIT;
        SET p_status_code = 200;
        SET p_message = 'Peminjaman berhasil diajukan dan menunggu persetujuan petugas.';
    END IF;
END$$

-- Prosedur 2: Pengembalian Alat & Perhitungan Denda
CREATE PROCEDURE sp_proses_pengembalian(
    IN p_kode_kembali VARCHAR(30),
    IN p_peminjaman_id INT,
    IN p_petugas_id INT,
    IN p_tgl_kembali DATE,
    IN p_kondisi_kembali VARCHAR(50),
    IN p_denda_kerusakan DECIMAL(10,2),
    IN p_catatan TEXT,
    OUT p_status_code INT,
    OUT p_total_denda_out DECIMAL(10,2),
    OUT p_message VARCHAR(255)
)
BEGIN
    DECLARE v_user_id INT;
    DECLARE v_tgl_rencana DATE;
    DECLARE v_tarif_harian DECIMAL(10,2);
    DECLARE v_hari_terlambat INT;
    DECLARE v_denda_telat DECIMAL(10,2);
    DECLARE v_total_denda DECIMAL(10,2);
    DECLARE v_alat_id INT;
    DECLARE v_jumlah INT;
    DECLARE v_status_bayar VARCHAR(20);

    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        SET p_status_code = 500;
        SET p_message = 'Gagal memproses pengembalian alat.';
    END;

    START TRANSACTION;

    -- Ambil data peminjaman
    SELECT user_id, tanggal_kembali_rencana 
    INTO v_user_id, v_tgl_rencana
    FROM peminjaman 
    WHERE id = p_peminjaman_id AND status IN ('disetujui', 'dipinjam')
    FOR UPDATE;

    IF v_user_id IS NULL THEN
        ROLLBACK;
        SET p_status_code = 404;
        SET p_message = 'Data peminjaman tidak ditemukan atau belum disetujui.';
    ELSE
        -- Ambil data detail & tarif denda alat
        SELECT dp.alat_id, dp.jumlah, a.tarif_denda_harian
        INTO v_alat_id, v_jumlah, v_tarif_harian
        FROM detail_peminjaman dp
        JOIN alat a ON dp.alat_id = a.id
        WHERE dp.peminjaman_id = p_peminjaman_id
        LIMIT 1;

        -- Hitung selisih hari & denda keterlambatan
        SET v_hari_terlambat = DATEDIFF(p_tgl_kembali, v_tgl_rencana);
        IF v_hari_terlambat < 0 THEN
            SET v_hari_terlambat = 0;
        END IF;

        SET v_denda_telat = v_hari_terlambat * v_tarif_harian;
        SET v_total_denda = v_denda_telat + p_denda_kerusakan;

        IF v_total_denda > 0 THEN
            SET v_status_bayar = 'belum_lunas';
        ELSE
            SET v_status_bayar = 'tidak_ada_denda';
        END IF;

        -- Update header peminjaman
        UPDATE peminjaman 
        SET status = 'selesai', 
            tanggal_kembali_aktual = p_tgl_kembali
        WHERE id = p_peminjaman_id;

        -- Update detail kondisi alat saat kembali
        UPDATE detail_peminjaman
        SET kondisi_saat_kembali = p_kondisi_kembali,
            denda_kerusakan = p_denda_kerusakan
        WHERE peminjaman_id = p_peminjaman_id;

        -- Pulihkan stok alat
        UPDATE alat 
        SET stok_tersedia = stok_tersedia + v_jumlah
        WHERE id = v_alat_id;

        -- Catat data pengembalian
        INSERT INTO pengembalian (
            kode_kembali, peminjaman_id, user_id, petugas_id, tanggal_kembali,
            hari_terlambat, denda_keterlambatan, denda_kerusakan, total_denda,
            status_pembayaran, catatan
        ) VALUES (
            p_kode_kembali, p_peminjaman_id, v_user_id, p_petugas_id, p_tgl_kembali,
            v_hari_terlambat, v_denda_telat, p_denda_kerusakan, v_total_denda,
            v_status_bayar, p_catatan
        );

        -- Log aktivitas
        INSERT INTO log_aktivitas (user_id, username, role, aksi, modul, detail)
        VALUES (p_petugas_id, 'petugas', 'petugas', 'PENGEMBALIAN', 'PENGEMBALIAN',
                CONCAT('Pengembalian id pinjam: ', p_peminjaman_id, ', denda: Rp ', v_total_denda));

        COMMIT;
        SET p_status_code = 200;
        SET p_total_denda_out = v_total_denda;
        SET p_message = 'Pengembalian alat berhasil diproses dan stok telah dipulihkan.';
    END IF;
END$$

DELIMITER ;

-- =============================================================================
-- 5. TRIGGERS (PEMICU OTOMATIS)
-- =============================================================================
DELIMITER $$

-- Trigger 1: Potong stok alat otomatis saat status peminjaman diubah menjadi 'disetujui' atau 'dipinjam'
CREATE TRIGGER trg_kurangi_stok_saat_disetujui
AFTER UPDATE ON peminjaman
FOR EACH ROW
BEGIN
    IF (OLD.status = 'menunggu' AND (NEW.status = 'disetujui' OR NEW.status = 'dipinjam')) THEN
        UPDATE alat a
        JOIN detail_peminjaman dp ON a.id = dp.alat_id
        SET a.stok_tersedia = a.stok_tersedia - dp.jumlah
        WHERE dp.peminjaman_id = NEW.id;
    END IF;
END$$

-- Trigger 2: Kembalikan stok alat otomatis jika peminjaman yang disetujui dibatalkan
CREATE TRIGGER trg_kembalikan_stok_saat_dibatalkan
AFTER UPDATE ON peminjaman
FOR EACH ROW
BEGIN
    IF ((OLD.status = 'disetujui' OR OLD.status = 'dipinjam') AND NEW.status = 'dibatalkan') THEN
        UPDATE alat a
        JOIN detail_peminjaman dp ON a.id = dp.alat_id
        SET a.stok_tersedia = a.stok_tersedia + dp.jumlah
        WHERE dp.peminjaman_id = NEW.id;
    END IF;
END$$

-- Trigger 3: Catat otomatis penambahan alat baru ke log aktivitas
CREATE TRIGGER trg_log_alat_baru
AFTER INSERT ON alat
FOR EACH ROW
BEGIN
    INSERT INTO log_aktivitas (aksi, modul, detail)
    VALUES ('CREATE', 'ALAT', CONCAT('Tambah alat baru: ', NEW.nama_alat, ' (Kode: ', NEW.kode_alat, '), Stok: ', NEW.stok_total));
END$$

DELIMITER ;

-- =============================================================================
-- 6. DATA AWAL (SEED DATA LENGKAP & SIAP DIUJI)
-- =============================================================================

-- Seed Users (Password plain / hash)
-- Default password: admin123, petugas123, peminjam123
INSERT INTO users (username, password, nama_lengkap, no_telepon, alamat, role, status) VALUES
('admin', 'admin123', 'Administrator Utama', '081234567890', 'Lab Komputer 1', 'admin', 'aktif'),
('petugas', 'petugas123', 'Budi Santoso (Petugas Lab)', '081234567891', 'Ruang Petugas Inventaris', 'petugas', 'aktif'),
('peminjam', 'peminjam123', 'Ahmad Dani (Siswa RPL)', '081234567892', 'Kelas XII RPL 1', 'peminjam', 'aktif'),
('siswa2', 'siswa123', 'Siti Rahma (Siswi RPL)', '081234567893', 'Kelas XII RPL 2', 'peminjam', 'aktif');

-- Seed Kategori Alat
INSERT INTO kategori (nama_kategori, deskripsi) VALUES
('Elektronik & IoT', 'Komponen mikrokontroler, sensor, kabel jumper, board ESP32/Arduino'),
('Komputer & Jaringan', 'Laptop praktik, Switch hub, router Mikrotik, crimping tool, LAN tester'),
('Audio Visual', 'Proyektor multimedia, kamera digital, mikrofon wireless, tripod'),
('Pertukangan & Mekanik', 'Solder listrik, multimeter digital, set obeng presisi, bor tangan');

-- Seed Alat
INSERT INTO alat (kode_alat, nama_alat, kategori_id, stok_total, stok_tersedia, kondisi, lokasi, spesifikasi, tarif_denda_harian) VALUES
('ALT-001', 'Arduino Uno R3 Starter Kit', 1, 15, 15, 'baik', 'Rak A-1', 'Board ATmega328P + Kabel USB + Breadboard + 30 Sensor', 5000.00),
('ALT-002', 'ESP32 NodeMCU Wi-Fi + BLE', 1, 20, 20, 'baik', 'Rak A-2', 'Dual Core 240MHz, 4MB Flash, Wi-Fi 802.11 b/g/n', 5000.00),
('ALT-003', 'Crimping Tool RJ45 CAT6', 2, 10, 10, 'baik', 'Kotak Alat Jaringan 1', 'Tang crimping profesional 3 in 1 untuk kabel UTP CAT5e/CAT6', 3000.00),
('ALT-004', 'Digital LAN Cable Tester', 2, 8, 8, 'baik', 'Kotak Alat Jaringan 1', 'Tester RJ45 / RJ11 dengan indikator LED 8 pin', 3000.00),
('ALT-005', 'Router MikroTik RB750r2 (hEX lite)', 2, 12, 12, 'baik', 'Lemari Server', '5 Port Fast Ethernet, RouterOS Level 4', 10000.00),
('ALT-006', 'Proyektor Epson EB-X500', 3, 4, 4, 'baik', 'Ruang Multimedia', '3600 Lumens, Resolusi XGA 1024x768, Port HDMI/VGA', 25000.00),
('ALT-007', 'Solder Listrik Digital 60W', 4, 10, 10, 'baik', 'Meja Praktik Solder', 'Suhu adjustable 200-450C dengan dudukan dan timah', 5000.00),
('ALT-008', 'Multimeter Digital Auto-Ranging', 4, 6, 6, 'baik', 'Meja Pengukuran', 'Pengukur tegangan AC/DC, resistansi, kontinuitas buzzer', 7500.00);

-- Seed Contoh Transaksi Peminjaman Selesai (Untuk Demo Laporan)
INSERT INTO peminjaman (kode_pinjam, user_id, tanggal_pinjam, tanggal_kembali_rencana, tanggal_kembali_aktual, status, keperluan, disetujui_oleh)
VALUES 
('PINJAM-20260901-001', 3, '2026-09-01', '2026-09-04', '2026-09-04', 'selesai', 'Praktik Uji Coba Jaringan LAN', 2);

INSERT INTO detail_peminjaman (peminjaman_id, alat_id, jumlah, kondisi_saat_pinjam, kondisi_saat_kembali)
VALUES (1, 3, 2, 'baik', 'baik');

INSERT INTO pengembalian (kode_kembali, peminjaman_id, user_id, petugas_id, tanggal_kembali, hari_terlambat, denda_keterlambatan, denda_kerusakan, total_denda, status_pembayaran, catatan)
VALUES ('KEMBALI-20260904-001', 1, 3, 2, '2026-09-04', 0, 0.00, 0.00, 0.00, 'tidak_ada_denda', 'Alat kembali tepat waktu dalam keadaan lengkap dan baik.');

-- Seed Contoh Transaksi Peminjaman Sedang Berjalan (Status Dipinjam)
INSERT INTO peminjaman (kode_pinjam, user_id, tanggal_pinjam, tanggal_kembali_rencana, status, keperluan, disetujui_oleh)
VALUES 
('PINJAM-20260910-002', 3, '2026-09-10', '2026-09-14', 'dipinjam', 'Praktik Proyek IoT Rumah Pintar', 2);

INSERT INTO detail_peminjaman (peminjaman_id, alat_id, jumlah, kondisi_saat_pinjam)
VALUES (2, 1, 1, 'baik');

-- Update stok alat ALT-001 karena sedang dipinjam
UPDATE alat SET stok_tersedia = stok_tersedia - 1 WHERE id = 1;

-- Seed Log Aktivitas Awal
INSERT INTO log_aktivitas (user_id, username, role, aksi, modul, detail) VALUES
(1, 'admin', 'admin', 'INIT', 'SISTEM', 'Inisialisasi basis data aplikasi peminjaman alat UKK'),
(2, 'petugas', 'petugas', 'APPROVAL', 'PEMINJAMAN', 'Menyetujui peminjaman PINJAM-20260901-001'),
(2, 'petugas', 'petugas', 'VERIFIKASI', 'PENGEMBALIAN', 'Memverifikasi pengembalian KEMBALI-20260904-001');

-- =============================================================================
-- 7. CONTOH IMPLEMENTASI TRANSAKSI ACID: COMMIT DAN ROLLBACK
-- =============================================================================

-- Contoh A: Transaksi Sukses dengan COMMIT
START TRANSACTION;
    INSERT INTO kategori (nama_kategori, deskripsi) 
    VALUES ('Robotika', 'Kit robot pemadam api, drone mini, dan sensor ultrasonik');
    
    INSERT INTO log_aktivitas (user_id, username, role, aksi, modul, detail)
    VALUES (1, 'admin', 'admin', 'CREATE', 'KATEGORI', 'Menambahkan kategori baru: Robotika');
COMMIT;

-- Contoh B: Transaksi dengan Kondisi Batal / ROLLBACK (Uji Integritas Data)
START TRANSACTION;
    -- Simulasi penambahan alat dengan stok negatif (tidak valid)
    -- Jika terjadi kegagalan validasi, perintah ROLLBACK dipanggil
    SELECT stok_tersedia FROM alat WHERE id = 999;
    -- Misal alat tidak ditemukan, batalkan transaksi:
ROLLBACK;

-- =============================================================================
-- SELESAI
-- =============================================================================
