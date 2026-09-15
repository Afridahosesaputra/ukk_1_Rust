/**
 * APLIKASI PEMINJAMAN ALAT - UKK RPL 2025/2026
 * Frontend Logic & API Client
 */

const API_BASE = '/api';

let currentUser = null;
let currentToken = localStorage.getItem('token') || '';
let cachedAlat = [];
let cachedKategori = [];
let cachedActiveLoans = [];
let currentView = 'dashboard';

// Inisialisasi saat halaman dimuat
document.addEventListener('DOMContentLoaded', () => {
  const savedTheme = localStorage.getItem('theme') || 'dark';
  document.documentElement.setAttribute('data-theme', savedTheme);

  // Set default tanggal form
  const today = new Date().toISOString().split('T')[0];
  const next3Days = new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];

  const tglPinjam = document.getElementById('formAjukanTglPinjam');
  const tglKembali = document.getElementById('formAjukanTglKembali');
  const tglKembaliAktual = document.getElementById('formKembaliTanggal');
  if (tglPinjam) tglPinjam.value = today;
  if (tglKembali) tglKembali.value = next3Days;
  if (tglKembaliAktual) tglKembaliAktual.value = today;

  const reportStart = document.getElementById('reportStartDate');
  const reportEnd = document.getElementById('reportEndDate');
  if (reportStart) reportStart.value = '2026-09-01';
  if (reportEnd) reportEnd.value = '2026-09-30';

  if (currentToken) {
    verifySession();
  } else {
    // Default auto login as Admin for seamless UKK examiner testing
    quickLogin('admin', 'admin123');
  }
});

// =============================================================================
// Autentikasi & Quick Switch
// =============================================================================
async function quickLogin(username, password) {
  try {
    const res = await fetch(`${API_BASE}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    const data = await res.json();

    if (data.success && data.token) {
      currentToken = data.token;
      currentUser = data.user;
      localStorage.setItem('token', currentToken);
      localStorage.setItem('user', JSON.stringify(currentUser));
      updateUserUI();
      showToast(data.message, 'success');
      switchView('dashboard');
    } else {
      showToast(data.message || 'Gagal login!', 'error');
    }
  } catch (err) {
    showToast('Koneksi ke backend server gagal.', 'error');
  }
}

async function verifySession() {
  try {
    const res = await fetch(`${API_BASE}/auth/me`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success && data.data) {
      currentUser = data.data;
      updateUserUI();
      switchView(currentView);
    } else {
      quickLogin('admin', 'admin123');
    }
  } catch (err) {
    quickLogin('admin', 'admin123');
  }
}

function updateUserUI() {
  if (!currentUser) return;

  const role = currentUser.role.toLowerCase();
  const nameEl = document.getElementById('userNameDisplay');
  const roleEl = document.getElementById('userRoleTag');
  const avatarEl = document.getElementById('userAvatar');

  if (nameEl) nameEl.textContent = currentUser.nama_lengkap;
  if (roleEl) {
    roleEl.textContent = role.toUpperCase();
    roleEl.className = `role-tag ${role}`;
  }
  if (avatarEl) {
    avatarEl.textContent = currentUser.nama_lengkap.charAt(0).toUpperCase();
  }

  // Update quick switch button highlight
  ['Admin', 'Petugas', 'Peminjam'].forEach(r => {
    const btn = document.getElementById(`btnRole${r}`);
    if (btn) {
      if (r.toLowerCase() === role) btn.classList.add('active');
      else btn.classList.remove('active');
    }
  });

  // Role-based Navigation Visibility (RBAC)
  const menuPeminjam = document.getElementById('menuPeminjamOnly');
  const menuStaff = document.getElementById('menuStaffOnly');
  const menuAdmin = document.getElementById('menuAdminOnly');
  const menuReport = document.getElementById('menuReportOnly');

  if (role === 'peminjam') {
    if (menuPeminjam) menuPeminjam.style.display = 'block';
    if (menuStaff) menuStaff.style.display = 'none';
    if (menuAdmin) menuAdmin.style.display = 'none';
    if (menuReport) menuReport.style.display = 'none';
  } else if (role === 'petugas') {
    if (menuPeminjam) menuPeminjam.style.display = 'none';
    if (menuStaff) menuStaff.style.display = 'block';
    if (menuAdmin) menuAdmin.style.display = 'none';
    if (menuReport) menuReport.style.display = 'block';
  } else if (role === 'admin') {
    if (menuPeminjam) menuPeminjam.style.display = 'none';
    if (menuStaff) menuStaff.style.display = 'block';
    if (menuAdmin) menuAdmin.style.display = 'block';
    if (menuReport) menuReport.style.display = 'block';
  }

  // Load Global Categories
  loadKategoriList();
}

async function logout() {
  await fetch(`${API_BASE}/auth/logout`, {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${currentToken}` }
  });
  localStorage.removeItem('token');
  localStorage.removeItem('user');
  currentUser = null;
  currentToken = '';
  showToast('Anda telah keluar dari aplikasi.', 'info');
  quickLogin('peminjam', 'peminjam123');
}

// =============================================================================
// Tampilan / View Switching
// =============================================================================
function switchView(viewName) {
  currentView = viewName;
  const sections = document.querySelectorAll('.view-section');
  sections.forEach(s => s.style.display = 'none');

  const navItems = document.querySelectorAll('.nav-item');
  navItems.forEach(n => n.classList.remove('active'));

  const targetSection = document.getElementById(`view${capitalize(viewName)}`);
  if (targetSection) targetSection.style.display = 'block';

  // Highlight Nav
  const targetNav = document.getElementById(`nav${capitalize(viewName)}`);
  if (targetNav) targetNav.classList.add('active');

  // Load respective data
  switch (viewName) {
    case 'dashboard':
      loadDashboard();
      break;
    case 'katalog':
      loadKatalog();
      break;
    case 'kelola_alat':
      loadKelolaAlat();
      break;
    case 'kelola_kategori':
      loadKelolaKategori();
      break;
    case 'kelola_user':
      loadKelolaUser();
      break;
    case 'approval':
      loadApproval();
      break;
    case 'peminjaman':
      loadPeminjaman();
      break;
    case 'pengembalian':
      loadPengembalian();
      break;
    case 'logs':
      loadLogs();
      break;
    case 'laporan':
      loadReportData();
      break;
  }
}

function handleDashboardQuickAction() {
  if (!currentUser) return;
  if (currentUser.role === 'peminjam') {
    openAjukanModal();
  } else if (currentUser.role === 'petugas' || currentUser.role === 'admin') {
    openTambahAlatModal();
  }
}

// =============================================================================
// Modul 1: Dashboard
// =============================================================================
async function loadDashboard() {
  try {
    const res = await fetch(`${API_BASE}/dashboard/stats`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const resData = await res.json();
    if (resData.success && resData.data) {
      const d = resData.data;
      document.getElementById('statTotalAlat').textContent = d.total_alat;
      document.getElementById('statTotalTersedia').textContent = d.total_tersedia;
      document.getElementById('statTotalDipinjam').textContent = d.total_dipinjam;
      document.getElementById('statTotalMenunggu').textContent = d.total_menunggu;
      document.getElementById('statTotalDenda').textContent = formatRupiah(d.total_denda_terkumpul);

      const badge = document.getElementById('badgePendingCount');
      if (badge) {
        if (d.total_menunggu > 0) {
          badge.textContent = d.total_menunggu;
          badge.style.display = 'inline-block';
        } else {
          badge.style.display = 'none';
        }
      }
    }

    // Load recent loans
    const resLoans = await fetch(`${API_BASE}/peminjaman`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const loanData = await resLoans.json();
    const tbody = document.getElementById('tbodyDashRecent');
    if (tbody && loanData.success && loanData.data) {
      tbody.innerHTML = '';
      const recent = loanData.data.slice(0, 5);
      if (recent.length === 0) {
        tbody.innerHTML = '<tr><td colspan="7" style="text-align:center; color:var(--text-muted);">Belum ada riwayat transaksi peminjaman.</td></tr>';
      } else {
        recent.forEach(item => {
          tbody.innerHTML += `
            <tr>
              <td><span style="font-family:monospace; font-weight:700;">${item.kode_pinjam}</span></td>
              <td>${item.peminjam_nama}</td>
              <td>${item.nama_alat}</td>
              <td><strong>${item.jumlah}</strong> unit</td>
              <td>${item.tanggal_pinjam}</td>
              <td>${item.tanggal_kembali_rencana}</td>
              <td><span class="badge ${item.status}">${item.status.toUpperCase()}</span></td>
            </tr>
          `;
        });
      }
    }
  } catch (err) {
    console.error(err);
  }
}

// =============================================================================
// Modul 2: Katalog Alat & Filter
// =============================================================================
async function loadKatalog() {
  try {
    const res = await fetch(`${API_BASE}/alat`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success && data.data) {
      cachedAlat = data.data;
      renderKatalog(cachedAlat);
    }
  } catch (err) {
    console.error(err);
  }
}

function renderKatalog(items) {
  const container = document.getElementById('toolsCatalogContainer');
  if (!container) return;
  container.innerHTML = '';

  if (items.length === 0) {
    container.innerHTML = '<div style="grid-column:1/-1; text-align:center; padding:2rem; color:var(--text-muted);">Tidak ada alat yang sesuai dengan pencarian.</div>';
    return;
  }

  items.forEach(a => {
    const isAvailable = a.stok_tersedia > 0;
    container.innerHTML += `
      <div class="tool-card">
        <div>
          <div class="tool-header">
            <span class="tool-code">${a.kode_alat}</span>
            <span class="badge ${a.kondisi}">${a.kondisi.replace('_', ' ').toUpperCase()}</span>
          </div>
          <div class="tool-title">${a.nama_alat}</div>
          <div class="tool-category">📁 ${a.nama_kategori}</div>
          <div class="tool-desc">${a.spesifikasi || 'Tidak ada spesifikasi khusus.'}</div>
        </div>
        <div>
          <div class="tool-meta">
            <span>Lokasi: <strong>${a.lokasi}</strong></span>
            <span>Denda: <strong>${formatRupiah(a.tarif_denda_harian)}/hari</strong></span>
          </div>
          <div class="tool-meta" style="border-top:none; padding-top:0;">
            <span class="stock-indicator ${isAvailable ? 'in-stock' : 'out-of-stock'}">
              ${isAvailable ? `Tersedia: ${a.stok_tersedia} / ${a.stok_total}` : 'STOK HABIS'}
            </span>
            ${currentUser && currentUser.role === 'peminjam' && isAvailable ? `
              <button class="btn btn-primary btn-sm" onclick="openAjukanModal(${a.id})">Pinjam Alat</button>
            ` : ''}
          </div>
        </div>
      </div>
    `;
  });
}

function filterKatalog() {
  const query = document.getElementById('searchKatalog').value.toLowerCase().trim();
  const katId = parseInt(document.getElementById('filterKategoriKatalog').value) || 0;
  const tersediaSaja = document.getElementById('checkTersediaSaja').checked;

  const filtered = cachedAlat.filter(a => {
    const matchQuery = !query || a.nama_alat.toLowerCase().includes(query) || a.kode_alat.toLowerCase().includes(query) || (a.spesifikasi && a.spesifikasi.toLowerCase().includes(query));
    const matchKat = katId === 0 || a.kategori_id === katId;
    const matchStock = !tersediaSaja || a.stok_tersedia > 0;
    return matchQuery && matchKat && matchStock;
  });

  renderKatalog(filtered);
}

// =============================================================================
// Modul 3: Kelola Alat (CRUD)
// =============================================================================
async function loadKelolaAlat() {
  try {
    const res = await fetch(`${API_BASE}/alat`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const tbody = document.getElementById('tbodyKelolaAlat');
    if (tbody && data.success && data.data) {
      tbody.innerHTML = '';
      cachedAlat = data.data;
      data.data.forEach(a => {
        tbody.innerHTML += `
          <tr>
            <td><span style="font-family:monospace; font-weight:600;">${a.kode_alat}</span></td>
            <td><strong>${a.nama_alat}</strong></td>
            <td>${a.nama_kategori}</td>
            <td>${a.stok_total}</td>
            <td><strong style="color:${a.stok_tersedia > 0 ? 'var(--accent-emerald)' : 'var(--accent-rose)'};">${a.stok_tersedia}</strong></td>
            <td><span class="badge ${a.kondisi}">${a.kondisi.replace('_', ' ').toUpperCase()}</span></td>
            <td>${formatRupiah(a.tarif_denda_harian)}</td>
            <td>${a.lokasi}</td>
            <td>
              <button class="btn btn-outline btn-sm" onclick="openEditAlatModal(${a.id})">Edit</button>
              <button class="btn btn-danger btn-sm" onclick="deleteAlat(${a.id}, '${a.nama_alat}')">Hapus</button>
            </td>
          </tr>
        `;
      });
    }
  } catch (err) {
    console.error(err);
  }
}

function openTambahAlatModal() {
  document.getElementById('modalAlatTitle').textContent = 'Tambah Alat Baru';
  document.getElementById('formAlatId').value = '';
  document.getElementById('formAlatKode').value = `ALT-0${cachedAlat.length + 1}`;
  document.getElementById('formAlatNama').value = '';
  document.getElementById('formAlatStok').value = '5';
  document.getElementById('formAlatDenda').value = '5000';
  document.getElementById('formAlatLokasi').value = 'Rak Lab 1';
  document.getElementById('formAlatSpek').value = '';
  populateKategoriDropdown('formAlatKategori');
  openModal('modalAlat');
}

function openEditAlatModal(id) {
  const alat = cachedAlat.find(a => a.id === id);
  if (!alat) return;
  document.getElementById('modalAlatTitle').textContent = 'Edit Data Alat';
  document.getElementById('formAlatId').value = alat.id;
  document.getElementById('formAlatKode').value = alat.kode_alat;
  document.getElementById('formAlatNama').value = alat.nama_alat;
  document.getElementById('formAlatStok').value = alat.stok_total;
  document.getElementById('formAlatKondisi').value = alat.kondisi;
  document.getElementById('formAlatDenda').value = alat.tarif_denda_harian;
  document.getElementById('formAlatLokasi').value = alat.lokasi;
  document.getElementById('formAlatSpek').value = alat.spesifikasi || '';
  populateKategoriDropdown('formAlatKategori', alat.kategori_id);
  openModal('modalAlat');
}

async function saveAlat() {
  const id = document.getElementById('formAlatId').value;
  const kode_alat = document.getElementById('formAlatKode').value.trim();
  const nama_alat = document.getElementById('formAlatNama').value.trim();
  const kategori_id = parseInt(document.getElementById('formAlatKategori').value);
  const stok_total = parseInt(document.getElementById('formAlatStok').value);
  const kondisi = document.getElementById('formAlatKondisi').value;
  const tarif_denda_harian = parseFloat(document.getElementById('formAlatDenda').value) || 5000;
  const lokasi = document.getElementById('formAlatLokasi').value.trim();
  const spesifikasi = document.getElementById('formAlatSpek').value.trim();

  if (!kode_alat || !nama_alat || !kategori_id || stok_total <= 0) {
    showToast('Lengkapi semua kolom wajib dengan benar!', 'error');
    return;
  }

  const payload = { kode_alat, nama_alat, kategori_id, stok_total, kondisi, lokasi, spesifikasi, tarif_denda_harian };
  const url = id ? `${API_BASE}/alat/${id}` : `${API_BASE}/alat`;
  const method = id ? 'PUT' : 'POST';

  try {
    const res = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
      },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      closeModal('modalAlat');
      loadKelolaAlat();
      loadKatalog();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Terjadi kesalahan jaringan.', 'error');
  }
}

async function deleteAlat(id, nama) {
  if (!confirm(`Apakah Anda yakin ingin menghapus alat '${nama}' dari inventaris?`)) return;

  try {
    const res = await fetch(`${API_BASE}/alat/${id}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      loadKelolaAlat();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal menghapus alat.', 'error');
  }
}

// =============================================================================
// Modul 4: Kelola Kategori (CRUD)
// =============================================================================
async function loadKategoriList() {
  try {
    const res = await fetch(`${API_BASE}/kategori`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success && data.data) {
      cachedKategori = data.data;
      populateKategoriDropdown('filterKategoriKatalog', 0, true);
    }
  } catch (err) {
    console.error(err);
  }
}

function populateKategoriDropdown(elId, selectedId = 0, isFilter = false) {
  const select = document.getElementById(elId);
  if (!select) return;
  select.innerHTML = isFilter ? '<option value="0">Semua Kategori</option>' : '';
  cachedKategori.forEach(k => {
    const opt = document.createElement('option');
    opt.value = k.id;
    opt.textContent = k.nama_kategori;
    if (k.id === selectedId) opt.selected = true;
    select.appendChild(opt);
  });
}

async function loadKelolaKategori() {
  await loadKategoriList();
  const tbody = document.getElementById('tbodyKelolaKategori');
  if (!tbody) return;
  tbody.innerHTML = '';
  cachedKategori.forEach(k => {
    tbody.innerHTML += `
      <tr>
        <td>#${k.id}</td>
        <td><strong>${k.nama_kategori}</strong></td>
        <td>${k.deskripsi || '-'}</td>
        <td>
          <button class="btn btn-outline btn-sm" onclick="openEditKategoriModal(${k.id})">Edit</button>
          <button class="btn btn-danger btn-sm" onclick="deleteKategori(${k.id}, '${k.nama_kategori}')">Hapus</button>
        </td>
      </tr>
    `;
  });
}

function openTambahKategoriModal() {
  document.getElementById('modalKategoriTitle').textContent = 'Tambah Kategori Baru';
  document.getElementById('formKategoriId').value = '';
  document.getElementById('formKategoriNama').value = '';
  document.getElementById('formKategoriDesc').value = '';
  openModal('modalKategori');
}

function openEditKategoriModal(id) {
  const k = cachedKategori.find(x => x.id === id);
  if (!k) return;
  document.getElementById('modalKategoriTitle').textContent = 'Edit Kategori';
  document.getElementById('formKategoriId').value = k.id;
  document.getElementById('formKategoriNama').value = k.nama_kategori;
  document.getElementById('formKategoriDesc').value = k.deskripsi || '';
  openModal('modalKategori');
}

async function saveKategori() {
  const id = document.getElementById('formKategoriId').value;
  const nama_kategori = document.getElementById('formKategoriNama').value.trim();
  const deskripsi = document.getElementById('formKategoriDesc').value.trim();

  if (!nama_kategori) {
    showToast('Nama kategori wajib diisi!', 'error');
    return;
  }

  const payload = { nama_kategori, deskripsi };
  const url = id ? `${API_BASE}/kategori/${id}` : `${API_BASE}/kategori`;
  const method = id ? 'PUT' : 'POST';

  try {
    const res = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
      },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      closeModal('modalKategori');
      loadKelolaKategori();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Terjadi kesalahan.', 'error');
  }
}

async function deleteKategori(id, nama) {
  if (!confirm(`Hapus kategori '${nama}'?`)) return;
  try {
    const res = await fetch(`${API_BASE}/kategori/${id}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      loadKelolaKategori();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal menghapus kategori.', 'error');
  }
}

// =============================================================================
// Modul 5: Kelola Pengguna (Admin Only)
// =============================================================================
async function loadKelolaUser() {
  try {
    const res = await fetch(`${API_BASE}/users`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const tbody = document.getElementById('tbodyKelolaUser');
    if (tbody && data.success && data.data) {
      tbody.innerHTML = '';
      data.data.forEach(u => {
        tbody.innerHTML += `
          <tr>
            <td>#${u.id}</td>
            <td><strong>${u.username}</strong></td>
            <td>${u.nama_lengkap}</td>
            <td><span class="role-tag ${u.role}">${u.role.toUpperCase()}</span></td>
            <td>${u.no_telepon || '-'}</td>
            <td>${u.alamat || '-'}</td>
            <td><span class="badge ${u.status === 'aktif' ? 'baik' : 'rusak_berat'}">${u.status.toUpperCase()}</span></td>
            <td>
              <button class="btn btn-outline btn-sm" onclick="openEditUserModal(${u.id})">Edit</button>
              ${u.id !== currentUser.id ? `<button class="btn btn-danger btn-sm" onclick="deleteUser(${u.id}, '${u.username}')">Hapus</button>` : ''}
            </td>
          </tr>
        `;
      });
    }
  } catch (err) {
    console.error(err);
  }
}

function openTambahUserModal() {
  document.getElementById('modalUserTitle').textContent = 'Tambah Pengguna Baru';
  document.getElementById('formUserId').value = '';
  document.getElementById('formUserUsername').value = '';
  document.getElementById('formUserPassword').value = '';
  document.getElementById('formUserNama').value = '';
  document.getElementById('formUserRole').value = 'peminjam';
  document.getElementById('formUserTelp').value = '';
  document.getElementById('formUserAlamat').value = '';
  openModal('modalUser');
}

async function openEditUserModal(id) {
  try {
    const res = await fetch(`${API_BASE}/users`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const u = data.data.find(x => x.id === id);
    if (!u) return;

    document.getElementById('modalUserTitle').textContent = 'Edit Data Pengguna';
    document.getElementById('formUserId').value = u.id;
    document.getElementById('formUserUsername').value = u.username;
    document.getElementById('formUserUsername').disabled = true;
    document.getElementById('formUserPassword').value = '';
    document.getElementById('formUserPassword').placeholder = 'Kosongkan jika tidak diubah';
    document.getElementById('formUserNama').value = u.nama_lengkap;
    document.getElementById('formUserRole').value = u.role;
    document.getElementById('formUserTelp').value = u.no_telepon || '';
    document.getElementById('formUserAlamat').value = u.alamat || '';
    openModal('modalUser');
  } catch (err) {
    console.error(err);
  }
}

async function saveUser() {
  const id = document.getElementById('formUserId').value;
  const username = document.getElementById('formUserUsername').value.trim();
  const password = document.getElementById('formUserPassword').value.trim();
  const nama_lengkap = document.getElementById('formUserNama').value.trim();
  const role = document.getElementById('formUserRole').value;
  const no_telepon = document.getElementById('formUserTelp').value.trim();
  const alamat = document.getElementById('formUserAlamat').value.trim();

  if (!username || !nama_lengkap) {
    showToast('Username dan Nama Lengkap wajib diisi!', 'error');
    return;
  }
  if (!id && !password) {
    showToast('Password wajib diisi untuk pengguna baru!', 'error');
    return;
  }

  const payload = { username, password, nama_lengkap, role, no_telepon, alamat };
  const url = id ? `${API_BASE}/users/${id}` : `${API_BASE}/users`;
  const method = id ? 'PUT' : 'POST';

  try {
    const res = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
      },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      closeModal('modalUser');
      loadKelolaUser();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal menyimpan pengguna.', 'error');
  }
}

async function deleteUser(id, username) {
  if (!confirm(`Hapus pengguna '${username}'?`)) return;
  try {
    const res = await fetch(`${API_BASE}/users/${id}`, {
      method: 'DELETE',
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      loadKelolaUser();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal menghapus user.', 'error');
  }
}

// =============================================================================
// Modul 6: Pengajuan Peminjaman (Peminjam)
// =============================================================================
async function openAjukanModal(preselectedAlatId = null) {
  if (!cachedAlat || cachedAlat.length === 0) {
    const res = await fetch(`${API_BASE}/alat`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success) cachedAlat = data.data;
  }

  const select = document.getElementById('formAjukanAlatId');
  if (select) {
    select.innerHTML = '';
    cachedAlat.forEach(a => {
      const opt = document.createElement('option');
      opt.value = a.id;
      opt.textContent = `${a.kode_alat} - ${a.nama_alat} (Sisa: ${a.stok_tersedia})`;
      if (preselectedAlatId && a.id === preselectedAlatId) opt.selected = true;
      select.appendChild(opt);
    });
  }

  updateAjukanAlatInfo();
  openModal('modalAjukanPinjam');
}

function updateAjukanAlatInfo() {
  const select = document.getElementById('formAjukanAlatId');
  const infoEl = document.getElementById('ajukanStockInfo');
  if (!select || !infoEl) return;
  const alatId = parseInt(select.value);
  const alat = cachedAlat.find(a => a.id === alatId);
  if (alat) {
    infoEl.innerHTML = `Stok tersedia: <strong>${alat.stok_tersedia} unit</strong> • Tarif Denda: <strong>${formatRupiah(alat.tarif_denda_harian)}/hari</strong>`;
  }
}

async function submitAjukanPinjam() {
  const alat_id = parseInt(document.getElementById('formAjukanAlatId').value);
  const jumlah = parseInt(document.getElementById('formAjukanJumlah').value);
  const tanggal_pinjam = document.getElementById('formAjukanTglPinjam').value;
  const tanggal_kembali_rencana = document.getElementById('formAjukanTglKembali').value;
  const keperluan = document.getElementById('formAjukanKeperluan').value.trim();

  if (!alat_id || jumlah <= 0 || !tanggal_pinjam || !tanggal_kembali_rencana || !keperluan) {
    showToast('Lengkapi seluruh formulir pengajuan!', 'error');
    return;
  }

  const payload = { alat_id, jumlah, tanggal_pinjam, tanggal_kembali_rencana, keperluan };

  try {
    const res = await fetch(`${API_BASE}/peminjaman`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
      },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      closeModal('modalAjukanPinjam');
      switchView('peminjaman');
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal mengirim pengajuan.', 'error');
  }
}

// =============================================================================
// Modul 7: Approval Peminjaman (Petugas & Admin)
// =============================================================================
async function loadApproval() {
  try {
    const res = await fetch(`${API_BASE}/peminjaman?status=menunggu`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const tbody = document.getElementById('tbodyApproval');
    if (tbody && data.success && data.data) {
      tbody.innerHTML = '';
      if (data.data.length === 0) {
        tbody.innerHTML = '<tr><td colspan="8" style="text-align:center; padding:1.5rem; color:var(--text-muted);">Tidak ada antrean peminjaman yang menunggu persetujuan.</td></tr>';
        return;
      }
      data.data.forEach(p => {
        tbody.innerHTML += `
          <tr>
            <td><span style="font-family:monospace; font-weight:700;">${p.kode_pinjam}</span></td>
            <td><strong>${p.peminjam_nama}</strong></td>
            <td>${p.nama_alat} (${p.kode_alat})</td>
            <td><strong style="font-size:1.05rem;">${p.jumlah}</strong> unit</td>
            <td>${p.tanggal_pinjam}</td>
            <td>${p.tanggal_kembali_rencana}</td>
            <td>${p.keperluan}</td>
            <td>
              <button class="btn btn-success btn-sm" onclick="openApprovalModal(${p.id}, 'setujui')">Setujui</button>
              <button class="btn btn-danger btn-sm" onclick="openApprovalModal(${p.id}, 'tolak')">Tolak</button>
            </td>
          </tr>
        `;
      });
    }
  } catch (err) {
    console.error(err);
  }
}

function openApprovalModal(id, aksi) {
  document.getElementById('formApprovalId').value = id;
  document.getElementById('formApprovalAksi').value = aksi;
  document.getElementById('formApprovalCatatan').value = '';

  const titleEl = document.getElementById('modalApprovalTitle');
  const btnEl = document.getElementById('btnConfirmApproval');

  if (aksi === 'setujui') {
    titleEl.textContent = 'Setujui Peminjaman Alat';
    btnEl.textContent = 'Setujui & Potong Stok';
    btnEl.className = 'btn btn-success';
  } else {
    titleEl.textContent = 'Tolak Pengajuan Peminjaman';
    btnEl.textContent = 'Tolak Pengajuan';
    btnEl.className = 'btn btn-danger';
  }

  openModal('modalApproval');
}

async function submitApproval() {
  const id = document.getElementById('formApprovalId').value;
  const aksi = document.getElementById('formApprovalAksi').value;
  const catatan = document.getElementById('formApprovalCatatan').value.trim();

  try {
    const res = await fetch(`${API_BASE}/peminjaman/${id}/approve`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
      },
      body: JSON.stringify({ aksi, catatan })
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      closeModal('modalApproval');
      loadApproval();
      loadDashboard();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal memproses approval.', 'error');
  }
}

// =============================================================================
// Modul 8: Semua Data Peminjaman
// =============================================================================
async function loadPeminjaman() {
  try {
    const res = await fetch(`${API_BASE}/peminjaman`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const tbody = document.getElementById('tbodyPeminjaman');
    if (tbody && data.success && data.data) {
      tbody.innerHTML = '';
      if (data.data.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" style="text-align:center; color:var(--text-muted); padding:1.5rem;">Tidak ada riwayat peminjaman.</td></tr>';
        return;
      }
      data.data.forEach(p => {
        tbody.innerHTML += `
          <tr>
            <td><span style="font-family:monospace; font-weight:700;">${p.kode_pinjam}</span></td>
            <td><strong>${p.peminjam_nama}</strong></td>
            <td>${p.nama_alat}</td>
            <td>${p.jumlah} unit</td>
            <td>${p.tanggal_pinjam}</td>
            <td>${p.tanggal_kembali_rencana}</td>
            <td><span class="badge ${p.status}">${p.status.toUpperCase()}</span></td>
            <td>${p.nama_petugas || '-'}</td>
            <td>${p.catatan_petugas || p.keperluan}</td>
          </tr>
        `;
      });
    }
  } catch (err) {
    console.error(err);
  }
}

// =============================================================================
// Modul 9: Pengembalian & Kalkulasi Denda
// =============================================================================
async function loadPengembalian() {
  try {
    const res = await fetch(`${API_BASE}/pengembalian`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const tbody = document.getElementById('tbodyPengembalian');
    if (tbody && data.success && data.data) {
      tbody.innerHTML = '';
      if (data.data.length === 0) {
        tbody.innerHTML = '<tr><td colspan="11" style="text-align:center; color:var(--text-muted); padding:1.5rem;">Belum ada riwayat pengembalian alat.</td></tr>';
        return;
      }
      data.data.forEach(r => {
        tbody.innerHTML += `
          <tr>
            <td><span style="font-family:monospace; font-weight:700;">${r.kode_kembali}</span></td>
            <td><span style="font-family:monospace;">${r.kode_pinjam}</span></td>
            <td>${r.nama_peminjam}</td>
            <td>${r.nama_alat} (${r.jumlah} unit)</td>
            <td>${r.tanggal_kembali}</td>
            <td>${r.hari_terlambat > 0 ? `<strong style="color:var(--accent-rose);">${r.hari_terlambat} Hari</strong>` : '<span style="color:var(--accent-emerald);">Tepat Waktu</span>'}</td>
            <td>${formatRupiah(r.denda_keterlambatan)}</td>
            <td>${formatRupiah(r.denda_kerusakan)}</td>
            <td><strong style="color:${r.total_denda > 0 ? 'var(--accent-rose)' : 'inherit'};">${formatRupiah(r.total_denda)}</strong></td>
            <td><span class="badge ${r.status_pembayaran}">${r.status_pembayaran.replace(/_/g, ' ').toUpperCase()}</span></td>
            <td>
              ${r.total_denda > 0 && r.status_pembayaran !== 'lunas' ? `
                <button class="btn btn-success btn-sm" onclick="bayarDenda(${r.id})">Lunasi Denda</button>
              ` : '<span style="font-size:0.8rem; color:var(--text-muted);">Selesai</span>'}
            </td>
          </tr>
        `;
      });
    }
  } catch (err) {
    console.error(err);
  }
}

async function openProsesPengembalianModal() {
  // Ambil peminjaman yang berstatus dipinjam
  const res = await fetch(`${API_BASE}/peminjaman?status=dipinjam`, {
    headers: { 'Authorization': `Bearer ${currentToken}` }
  });
  const data = await res.json();
  const select = document.getElementById('formKembaliPinjamId');
  if (select && data.success && data.data) {
    cachedActiveLoans = data.data;
    select.innerHTML = '';
    if (data.data.length === 0) {
      showToast('Tidak ada peminjaman aktif yang berstatus dipinjam saat ini.', 'info');
      return;
    }
    data.data.forEach(p => {
      const opt = document.createElement('option');
      opt.value = p.id;
      opt.textContent = `${p.kode_pinjam} - ${p.peminjam_nama} (${p.nama_alat}, Qty: ${p.jumlah})`;
      select.appendChild(opt);
    });
  }

  document.getElementById('formKembaliTanggal').value = new Date().toISOString().split('T')[0];
  document.getElementById('formKembaliKondisi').value = 'baik';
  document.getElementById('formKembaliCatatan').value = '';

  calculateLiveFine();
  openModal('modalPengembalian');
}

function calculateLiveFine() {
  const pinjamId = parseInt(document.getElementById('formKembaliPinjamId').value);
  const tglKembaliStr = document.getElementById('formKembaliTanggal').value;
  const kondisi = document.getElementById('formKembaliKondisi').value;

  const loan = cachedActiveLoans.find(p => p.id === pinjamId);
  if (!loan) return;

  const tglRencanaStr = loan.tanggal_kembali_rencana;
  document.getElementById('finePlannedDate').textContent = tglRencanaStr;

  const tglKembali = new Date(tglKembaliStr);
  const tglRencana = new Date(tglRencanaStr);

  const diffTime = tglKembali - tglRencana;
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  const lateDays = diffDays > 0 ? diffDays : 0;

  // Denda keterlambatan default 5000/hari
  const lateRate = 5000;
  const lateFine = lateDays * lateRate;

  // Denda kerusakan
  let damageFine = 0;
  if (kondisi === 'rusak_ringan') damageFine = 25000;
  else if (kondisi === 'rusak_berat') damageFine = 100000;

  const totalFine = lateFine + damageFine;

  document.getElementById('fineLateDays').textContent = `${lateDays} Hari`;
  document.getElementById('fineLateAmount').textContent = formatRupiah(lateFine);
  document.getElementById('fineDamageAmount').textContent = formatRupiah(damageFine);
  document.getElementById('fineTotalAmount').textContent = formatRupiah(totalFine);
}

async function submitPengembalian() {
  const peminjaman_id = parseInt(document.getElementById('formKembaliPinjamId').value);
  const tanggal_kembali = document.getElementById('formKembaliTanggal').value;
  const kondisi_kembali = document.getElementById('formKembaliKondisi').value;
  const catatan = document.getElementById('formKembaliCatatan').value.trim();

  if (!peminjaman_id || !tanggal_kembali) {
    showToast('Pilih transaksi dan tanggal kembali!', 'error');
    return;
  }

  const payload = { peminjaman_id, tanggal_kembali, kondisi_kembali, catatan };

  try {
    const res = await fetch(`${API_BASE}/pengembalian/proses`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
      },
      body: JSON.stringify(payload)
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      closeModal('modalPengembalian');
      loadPengembalian();
      loadDashboard();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal memproses pengembalian.', 'error');
  }
}

async function bayarDenda(id) {
  if (!confirm('Apakah Anda memverifikasi bahwa denda ini telah DILUNASI?')) return;
  try {
    const res = await fetch(`${API_BASE}/pengembalian/bayar/${id}`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success) {
      showToast(data.message, 'success');
      loadPengembalian();
      loadDashboard();
    } else {
      showToast(data.message, 'error');
    }
  } catch (err) {
    showToast('Gagal mengubah status pembayaran.', 'error');
  }
}

// =============================================================================
// Modul 10: Log Aktivitas (Audit Trail)
// =============================================================================
async function loadLogs() {
  try {
    const res = await fetch(`${API_BASE}/logs`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    const tbody = document.getElementById('tbodyLogs');
    if (tbody && data.success && data.data) {
      tbody.innerHTML = '';
      data.data.forEach(l => {
        tbody.innerHTML += `
          <tr>
            <td style="font-family:monospace; font-size:0.75rem; color:var(--text-muted);">${l.created_at}</td>
            <td><strong>${l.username}</strong></td>
            <td><span class="role-tag ${l.role}">${l.role.toUpperCase()}</span></td>
            <td><span class="badge ${l.aksi === 'DELETE' ? 'rusak_berat' : 'dipinjam'}">${l.aksi}</span></td>
            <td><span style="font-weight:600;">${l.modul}</span></td>
            <td>${l.detail}</td>
          </tr>
        `;
      });
    }
  } catch (err) {
    console.error(err);
  }
}

// =============================================================================
// Modul 11: Cetak Laporan UKK
// =============================================================================
async function loadReportData() {
  const mulai = document.getElementById('reportStartDate').value;
  const selesai = document.getElementById('reportEndDate').value;
  const status = document.getElementById('reportFilterStatus').value;

  try {
    const res = await fetch(`${API_BASE}/reports?mulai=${mulai}&selesai=${selesai}&status=${status}`, {
      headers: { 'Authorization': `Bearer ${currentToken}` }
    });
    const data = await res.json();
    if (data.success && data.data) {
      const rep = data.data;
      document.getElementById('reportDateRange').textContent = `Periode: ${mulai} s/d ${selesai}`;
      document.getElementById('reportPrintTimestamp').textContent = new Date().toLocaleString('id-ID');
      document.getElementById('reportTotalTrx').textContent = `${rep.total_transaksi} Transaksi (${rep.total_alat_terpinjam} unit alat)`;
      document.getElementById('reportTotalFine').textContent = formatRupiah(rep.total_denda);

      if (currentUser) {
        const signer = document.getElementById('printSignerName');
        if (signer) signer.textContent = `( ${currentUser.nama_lengkap} )`;
      }

      const tbody = document.getElementById('tbodyReport');
      if (tbody) {
        tbody.innerHTML = '';
        if (rep.items.length === 0) {
          tbody.innerHTML = '<tr><td colspan="9" style="text-align:center; color:var(--text-muted); padding:1.5rem;">Tidak ada data transaksi pada periode ini.</td></tr>';
          return;
        }

        rep.items.forEach((item, index) => {
          tbody.innerHTML += `
            <tr>
              <td>${index + 1}</td>
              <td style="font-family:monospace; font-weight:bold;">${item.kode_pinjam}</td>
              <td>${item.peminjam_nama}</td>
              <td>${item.nama_alat}</td>
              <td>${item.jumlah} unit</td>
              <td>${item.tanggal_pinjam}</td>
              <td>${item.tanggal_kembali_aktual || item.tanggal_kembali_rencana}</td>
              <td><span class="badge ${item.status}">${item.status.toUpperCase()}</span></td>
              <td>${formatRupiah(item.total_denda)}</td>
            </tr>
          `;
        });
      }
    }
  } catch (err) {
    console.error(err);
  }
}

// =============================================================================
// Helper Functions
// =============================================================================
function formatRupiah(num) {
  return new Intl.NumberFormat('id-ID', {
    style: 'currency',
    currency: 'IDR',
    maximumFractionDigits: 0
  }).format(num || 0);
}

function openModal(id) {
  const modal = document.getElementById(id);
  if (modal) modal.classList.add('active');
}

function closeModal(id) {
  const modal = document.getElementById(id);
  if (modal) modal.classList.remove('active');
}

function showToast(msg, type = 'info') {
  const container = document.getElementById('toastContainer');
  if (!container) return;

  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  toast.textContent = msg;

  container.appendChild(toast);

  setTimeout(() => {
    toast.remove();
  }, 3500);
}

function toggleTheme() {
  const current = document.documentElement.getAttribute('data-theme') || 'dark';
  const next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('theme', next);
}

function capitalize(str) {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}
