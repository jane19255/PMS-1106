const nationalities = [
  "Singapore",
  "Malaysia",
  "Thailand",
  "Indonesia",
  "Afghan",
  "Albanian",
  "Algerian",
  "American",
  "Andorran",
  "Angolan",
  "Argentine",
  "Armenian",
  "Australian",
  "Austrian",
  "Azerbaijani",
  "Bahraini",
  "Bangladeshi",
  "Barbadian",
  "Belarusian",
  "Belgian",
  "Belizean",
  "Beninese",
  "Bhutanese",
  "Bolivian",
  "Bosnian",
  "Botswanan",
  "Brazilian",
  "British",
  "Bruneian",
  "Bulgarian",
  "Burkinabe",
  "Burmese",
  "Burundian",
  "Cambodian",
  "Cameroonian",
  "Canadian",
  "Cape Verdean",
  "Central African",
  "Chadian",
  "Chilean",
  "Chinese",
  "Colombian",
  "Comoran",
  "Congolese",
  "Costa Rican",
  "Croatian",
  "Cuban",
  "Cypriot",
  "Czech",
  "Danish",
  "Djiboutian",
  "Dominican",
  "Dutch",
  "East Timorese",
  "Ecuadorian",
  "Egyptian",
  "Emirian",
  "Equatorial Guinean",
  "Eritrean",
  "Estonian",
  "Ethiopian",
  "Fijian",
  "Filipino",
  "Finnish",
  "French",
  "Gabonese",
  "Gambian",
  "Georgian",
  "German",
  "Ghanaian",
  "Greek",
  "Grenadian",
  "Guatemalan",
  "Guinean",
  "Guyanese",
  "Haitian",
  "Honduran",
  "Hungarian",
  "Icelandic",
  "Indian",
  "Iranian",
  "Iraqi",
  "Irish",
  "Israeli",
  "Italian",
  "Ivorian",
  "Jamaican",
  "Japanese",
  "Jordanian",
  "Kazakhstani",
  "Kenyan",
  "Kuwaiti",
  "Kyrgyz",
  "Laotian",
  "Latvian",
  "Lebanese",
  "Liberian",
  "Libyan",
  "Liechtenstein",
  "Lithuanian",
  "Luxembourg",
  "Macedonian",
  "Malagasy",
  "Malawian",
  "Maldivian",
  "Maltese",
  "Marshallese",
  "Mauritanian",
  "Mauritian",
  "Mexican",
  "Micronesian",
  "Moldovan",
  "Monacan",
  "Mongolian",
  "Montenegrin",
  "Moroccan",
  "Mozambican",
  "Namibian",
  "Nepalese",
  "New Zealander",
  "Nicaraguan",
  "Nigerian",
  "Nigerien",
  "North Korean",
  "Norwegian",
  "Omani",
  "Pakistani",
  "Palauan",
  "Panamanian",
  "Paraguayan",
  "Peruvian",
  "Polish",
  "Portuguese",
  "Qatari",
  "Romanian",
  "Russian",
  "Rwandan",
  "Saint Lucian",
  "Salvadoran",
  "Samoan",
  "Sao Tomean",
  "Saudi",
  "Scottish",
  "Senegalese",
  "Serbian",
  "Seychellois",
  "Sierra Leonean",
  "Slovak",
  "Slovenian",
  "Solomon Islander",
  "Somali",
  "South African",
  "South Korean",
  "Spanish",
  "Sri Lankan",
  "Sudanese",
  "Surinamese",
  "Swazi",
  "Swedish",
  "Swiss",
  "Syrian",
  "Taiwanese",
  "Tajik",
  "Tanzanian",
  "Togolese",
  "Tongan",
  "Trinidadian",
  "Tunisian",
  "Turkish",
  "Tuvaluan",
  "Ugandan",
  "Ukrainian",
  "Uruguayan",
  "Uzbek",
  "Venezuelan",
  "Vietnamese",
  "Yemeni",
  "Zambian",
  "Zimbabwean",
];

function populateNationalityDropdown(elementId) {
  const select = document.getElementById(elementId);
  if (!select) return;
  select.innerHTML =
    '<option value="" disabled selected>Select Nationality</option>';

  nationalities.forEach((nation) => {
    const option = document.createElement("option");
    option.value = nation;
    option.textContent = nation;
    select.appendChild(option);
  });
}

let patients = [];
let currentUser = null;
let currentUserPermissions = {
  canViewPatients: false,
  canCreatePatients: false,
  canEditPatients: false,
  canDeletePatients: false,
};

// Visits should eventually come from the database too.
// Keep this empty so the Patients page does not display fake PAT-001 visit data.
let allVisits = [];

async function loadCurrentUser() {
  try {
    const response = await fetch("/api/me", {
      method: "GET",
      headers: { Accept: "application/json" },
    });

    if (!response.ok) {
      throw new Error(await response.text());
    }

    currentUser = await response.json();
    currentUserPermissions = currentUser.permissions || currentUserPermissions;
    applyRoleBasedUi();
  } catch (error) {
    console.error("Failed to load current user role:", error);
    currentUser = null;
    currentUserPermissions = {
      canViewPatients: false,
      canCreatePatients: false,
      canEditPatients: false,
      canDeletePatients: false,
    };
    applyRoleBasedUi();
  }
}

function applyRoleBasedUi() {
  const addPatientButton = document.getElementById("addPatientBtn");
  if (addPatientButton && !currentUserPermissions.canCreatePatients) {
    addPatientButton.style.display = "none";
  }

  if (!currentUserPermissions.canEditPatients) {
    document.querySelectorAll("[data-edit-patient]").forEach((el) => {
      el.style.display = "none";
    });
  }

  if (!currentUserPermissions.canDeletePatients) {
    document.querySelectorAll("[data-delete-patient]").forEach((el) => {
      el.style.display = "none";
    });
  }
}

function formatDate(dateString) {
  if (!dateString) return "N.A.";
  const date = new Date(dateString);
  if (isNaN(date)) return dateString;
  const options = { day: "2-digit", month: "short", year: "numeric" };
  return date.toLocaleDateString("en-GB", options);
}

class PatientPagination {
  constructor(config) {
    this.data = config.data || [];
    this.rowsPerPage = config.rowsPerPage || 3;
    this.tbodyId = config.tbodyId;
    this.pageInfoId = config.pageInfoId;
    this.pageSelectId = config.pageSelectId;
    this.prevBtnId = config.prevBtnId;
    this.nextBtnId = config.nextBtnId;
    this.renderRow = config.renderRow;
    this.currentPage = 1;
  }

  get paginatedData() {
    const start = (this.currentPage - 1) * this.rowsPerPage;
    const end = start + parseInt(this.rowsPerPage);
    return this.data.slice(start, end);
  }

  get totalPages() {
    return Math.ceil(this.data.length / this.rowsPerPage) || 1;
  }

  renderTable() {
    const tbody = document.getElementById(this.tbodyId);
    if (!tbody) return;
    tbody.innerHTML = "";

    const currentData = this.paginatedData;
    if (currentData.length === 0) {
      tbody.innerHTML =
        "<tr><td colspan='7' style='text-align: center; padding: 20px;'>No patient records found in database.</td></tr>";
    } else {
      currentData.forEach((item, index) => {
        tbody.innerHTML += this.renderRow(item, index);
      });
    }
    this.updateControls();
  }

  updateControls() {
    const pageInfo = document.getElementById(this.pageInfoId);
    const pageSelect = document.getElementById(this.pageSelectId);
    const prevBtn = document.getElementById(this.prevBtnId);
    const nextBtn = document.getElementById(this.nextBtnId);

    if (pageInfo) {
      const start =
        this.data.length === 0
          ? 0
          : (this.currentPage - 1) * this.rowsPerPage + 1;
      const end = Math.min(
        this.currentPage * this.rowsPerPage,
        this.data.length,
      );
      pageInfo.innerText = `Showing ${start} to ${end} of ${this.data.length} entries`;
    }

    if (pageSelect) {
      pageSelect.innerHTML = "";
      for (let i = 1; i <= this.totalPages; i++) {
        const option = document.createElement("option");
        option.value = i;
        option.innerText = i;
        if (i === this.currentPage) option.selected = true;
        pageSelect.appendChild(option);
      }
    }

    if (prevBtn) prevBtn.disabled = this.currentPage === 1;
    if (nextBtn)
      nextBtn.disabled =
        this.currentPage === this.totalPages || this.totalPages === 0;
  }

  prev() {
    if (this.currentPage > 1) {
      this.currentPage--;
      this.renderTable();
    }
  }

  next() {
    if (this.currentPage < this.totalPages) {
      this.currentPage++;
      this.renderTable();
    }
  }

  goToPage(page) {
    this.currentPage = parseInt(page);
    this.renderTable();
  }

  changeRowsPerPage(rows) {
    this.rowsPerPage = parseInt(rows);
    this.currentPage = 1;
    this.renderTable();
  }
}

function renderPatientRow(patient, index) {
  const safePatientId = String(patient.id).replace(/'/g, "\\'");
  const billingUrl = `/billing?patient_id=${encodeURIComponent(patient.id)}`;
  const editAction = currentUserPermissions.canEditPatients
    ? `
            <div class="has-tooltip" data-edit-patient>
                <i class="edit fa-solid fa-pen-to-square" onclick="editPatient(${index})"></i>
                <span class="tooltip-text">Edit Details</span>
            </div>`
    : "";

  const deleteAction = currentUserPermissions.canDeletePatients
    ? `
            <div class="has-tooltip" data-delete-patient>
                <i class="delete fa-solid fa-trash" style="color: #ef4444;" onclick="removePatient('${safePatientId}')"></i>
                <span class="tooltip-text">Delete Patient</span>
            </div>`
    : "";

  return `
    <tr class="hover:bg-slate-50">
        <td>${patient.id}</td>
        <td><a class="navigation" href="${billingUrl}">${patient.firstName + " " + patient.lastName}</a></td>
        <td>${patient.gender}</td>
        <td>${formatDate(patient.dob)}</td>
        <td>${patient.phone}</td>
        <td><span class="status ${(patient.status || "").toLowerCase()}">${patient.status}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewPatient(${index})"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            ${editAction}
            ${deleteAction}
        </td>
    </tr>`;
}

function createInvoiceForSelectedPatient() {
  const patientId = document.getElementById("view-pid")?.innerText.trim();
  if (patientId) {
    window.location.href = `/billing?patient_id=${encodeURIComponent(patientId)}`;
  }
}

const pagination = new PatientPagination({
  data: [],
  rowsPerPage: 3,
  tbodyId: "patientTableBody",
  pageInfoId: "pageInfo",
  pageSelectId: "pageSelect",
  prevBtnId: "prevBtn",
  nextBtnId: "nextBtn",
  renderRow: renderPatientRow,
});

function viewPatient(index) {
  const p = pagination.paginatedData[index];

  document.getElementById("view-pid").innerText = p.id;
  document.getElementById("view-fullname").innerText =
    p.firstName + " " + p.lastName;
  document.getElementById("view-gender").innerText = p.gender;
  document.getElementById("view-dob").innerText = formatDate(p.dob);
  document.getElementById("view-nric").innerText = p.nric;
  document.getElementById("view-nationality").innerText = p.nationality;
  document.getElementById("view-phone").innerText = p.phone;
  document.getElementById("view-email").innerText = p.email;
  document.getElementById("view-address").innerText = p.address;
  document.getElementById("view-emergency").innerText =
    p.emergencyName + " | " + p.emergencyPhone;
  document.getElementById("view-allergies").innerText = p.allergies || "N.A.";
  document.getElementById("view-medications").innerText =
    p.medications || "N.A.";
  document.getElementById("view-conditions").innerText = p.conditions || "N.A.";

  const statusEl = document.getElementById("view-status");
  statusEl.innerText = p.status;
  statusEl.className = "status " + p.status.toLowerCase();

  const { upcoming, past } = categorizeVisits(p.visits);

  const upEl = document.getElementById("view-upcoming-visits");
  upEl.innerHTML = upcoming.length
    ? upcoming
        .map(
          (v) => `
    <div class="card" onclick="window.location.href='/appointments'">
        <div class="top"><div class="purpose">${v.purpose}</div><div class="date">${formatDate(v.date)}</div></div>
        <div class="summary">${v.summary}</div>
        <div class="navigation"><i class="fa-solid fa-calendar-check"></i>Go to Appointment</div>
    </div>`,
        )
        .join("")
    : "<div class='description'>No upcoming visits</div>";

  const pastEl = document.getElementById("view-past-visits");
  pastEl.innerHTML = past.length
    ? past
        .map(
          (v) => `
    <div class="card" onclick="window.location.href='/medical-records'">
        <div class="top"><div class="purpose">${v.purpose}</div><div class="date">${formatDate(v.date)}</div></div>
        <div class="summary">${v.summary}</div>
        <div class="navigation"><i class="fa-solid fa-file-medical"></i>View Record</div>
    </div>`,
        )
        .join("")
    : "<div class='description'>No past visits</div>";

  openModal("detailsModal");
}

function editPatient(index) {
  const p = pagination.paginatedData[index];
  const fullIndex = patients.findIndex((x) => x.id === p.id);

  document.getElementById("edit-index").value = fullIndex;
  document.getElementById("edit-firstName").value = p.firstName;
  document.getElementById("edit-lastName").value = p.lastName;
  document.getElementById("edit-dob").value = p.dob;

  document.getElementById("edit-gender").value = p.gender.toLowerCase();

  document.getElementById("edit-nric").value = p.nric;
  document.getElementById("edit-nationality").value = p.nationality;
  document.getElementById("edit-phone").value = p.phone;
  document.getElementById("edit-email").value = p.email;
  document.getElementById("edit-emergencyName").value = p.emergencyName;
  document.getElementById("edit-emergencyPhone").value = p.emergencyPhone;
  document.getElementById("edit-address").value = p.address;
  document.getElementById("edit-allergies").value = p.allergies;
  document.getElementById("edit-medications").value = p.medications;
  document.getElementById("edit-conditions").value = p.conditions;

  openModal("editPatientModal");
}

function groupPatientVisits(patients, allVisits) {
  return patients.map((patient) => {
    const patientVisits = allVisits.filter(
      (visit) => visit.patientId === patient.id,
    );
    return { ...patient, visits: patientVisits || [] };
  });
}

function categorizeVisits(visits) {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const upcoming = [];
  const past = [];

  visits.forEach((v) => {
    const d = new Date(v.date);
    d.setHours(0, 0, 0, 0);
    d >= today ? upcoming.push(v) : past.push(v);
  });

  upcoming.sort((a, b) => new Date(a.date) - new Date(b.date));
  past.sort((a, b) => new Date(b.date) - new Date(a.date));
  return { upcoming, past };
}

function applySearch() {
  const keyword =
    document.getElementById("searchInput")?.value.toLowerCase() || "";
  return patients.filter(
    (p) =>
      p.id.toLowerCase().includes(keyword) ||
      (p.nric || "").toLowerCase().includes(keyword) ||
      p.firstName.toLowerCase().includes(keyword) ||
      p.lastName.toLowerCase().includes(keyword) ||
      (p.email || "").toLowerCase().includes(keyword) ||
      p.phone.includes(keyword),
  );
}

function applyFilter(list) {
  const gender = document.getElementById("filter-gender")?.value || "";
  const status = document.getElementById("filter-status")?.value || "";
  return list.filter((item) => {
    let match = true;
    if (gender) match = match && item.gender === gender;
    if (status) match = match && item.status === status;
    return match;
  });
}

function applySort(list) {
  const sortBy = document.getElementById("sortBy")?.value || "id";
  const sorted = [...list];
  sorted.sort((a, b) => {
    if (sortBy === "name")
      return (a.firstName + a.lastName).localeCompare(b.firstName + b.lastName);
    if (sortBy === "dob") return new Date(a.dob) - new Date(b.dob);
    if (sortBy === "status") return a.status.localeCompare(b.status);
    return a.id.localeCompare(b.id);
  });
  return sorted;
}

function refreshPatientList() {
  let result = applySearch();
  result = applyFilter(result);
  result = applySort(result);

  const finalData = groupPatientVisits(result, allVisits);
  pagination.data = finalData;
  pagination.currentPage = 1;
  pagination.renderTable();
}

function updatePatientSummaryCards(total, newThisMonth, active, inactive) {
  const totalEl = document.querySelector("#totalPatients .number");
  const newEl = document.querySelector("#newPatients .number");
  const activeEl = document.querySelector("#activePatients .number");
  const inactiveEl = document.querySelector("#inactivePatients .number");

  if (totalEl) totalEl.innerText = total;
  if (newEl) newEl.innerText = newThisMonth;
  if (activeEl) activeEl.innerText = active;
  if (inactiveEl) inactiveEl.innerText = inactive;
}

async function loadData() {
  updatePatientSummaryCards("...", "...", "...", "...");

  try {
    const response = await fetch("/api/patients", {
      method: "GET",
      headers: { Accept: "application/json" },
    });

    if (!response.ok) {
      throw new Error(await response.text());
    }

    const loadedPatients = await response.json();
    let activeCount = 0;
    let inactiveCount = 0;
    let newThisMonthCount = 0;

    const now = new Date();
    const currentMonth = now.getMonth();
    const currentYear = now.getFullYear();

    loadedPatients.forEach((patient) => {
      if ((patient.status || "").toLowerCase() === "active") {
        activeCount++;
      } else {
        inactiveCount++;
      }

      if (patient.createdAt) {
        const createdAt = new Date(patient.createdAt);
        if (
          !Number.isNaN(createdAt.getTime()) &&
          createdAt.getMonth() === currentMonth &&
          createdAt.getFullYear() === currentYear
        ) {
          newThisMonthCount++;
        }
      }

      patient.visits = patient.visits || [];
    });

    patients = loadedPatients;
    updatePatientSummaryCards(
      patients.length,
      newThisMonthCount,
      activeCount,
      inactiveCount,
    );
    refreshPatientList();
  } catch (error) {
    console.error("Error fetching patients from Supabase API:", error);
    patients = [];
    pagination.data = [];
    pagination.currentPage = 1;
    pagination.renderTable();
    updatePatientSummaryCards(0, 0, 0, 0);
    showToast(
      "Failed to load patients from database. Check your Supabase key in .env.",
    );
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  populateNationalityDropdown("add-nationality");
  populateNationalityDropdown("edit-nationality");

  const today = new Date().toLocaleDateString("en-CA");
  const dateInputs = document.querySelectorAll('input[type="date"]');
  dateInputs.forEach((input) => {
    input.setAttribute("max", today);
  });

  await loadCurrentUser();
  await loadData();
});

async function submitNewPatient(button) {
  if (!currentUserPermissions.canCreatePatients) {
    showToast("You do not have permission to register patients.");
    return;
  }

  if (!verifyInput(button)) return;

  const patientData = {
    first_name: document.getElementById("reg-firstname").value,
    last_name: document.getElementById("reg-lastname").value,
    dob: document.getElementById("reg-dob").value,
    gender: document.getElementById("reg-gender").value,
    nric: document.getElementById("reg-nric").value,
    nationality: document.getElementById("add-nationality").value,
    phone: document.getElementById("reg-phone").value,
    email: document.getElementById("reg-email").value,
    emergency_name: document.getElementById("reg-emergencyname")?.value || null,
    emergency_phone:
      document.getElementById("reg-emergencyphone")?.value || null,
    address: document.getElementById("reg-address")?.value || null,
    allergies: document.getElementById("reg-allergies")?.value || null,
    medications: document.getElementById("reg-medications")?.value || null,
    conditions: document.getElementById("reg-conditions")?.value || null,
  };

  try {
    const response = await fetch("/api/patients/new", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(patientData),
    });

    if (response.ok) {
      showToast("New patient added successfully!");
      closeModal(button);
      clearInput(button);
      loadData();
    } else {
      const message = await response.text();
      showToast(message || "Error saving patient.", "error");
    }
  } catch (error) {
    console.error("Failed to submit patient:", error);
    showToast("Network error.", "error");
  }
}

async function saveEditedPatient(button) {
  if (!currentUserPermissions.canEditPatients) {
    showToast("You do not have permission to edit patients.");
    return;
  }

  const index = document.getElementById("edit-index").value;
  const patientId = patients[index].id;

  const updatedData = {
    firstName: document.getElementById("edit-firstName").value,
    lastName: document.getElementById("edit-lastName").value,
    dob: document.getElementById("edit-dob").value,
    gender: document.getElementById("edit-gender").value,
    nric: document.getElementById("edit-nric").value,
    nationality: document.getElementById("edit-nationality").value,
    phone: document.getElementById("edit-phone").value,
    email: document.getElementById("edit-email").value,
    emergencyName: document.getElementById("edit-emergencyName").value,
    emergencyPhone: document.getElementById("edit-emergencyPhone").value,
    address: document.getElementById("edit-address").value,
    allergies: document.getElementById("edit-allergies").value,
    medications: document.getElementById("edit-medications").value,
    conditions: document.getElementById("edit-conditions").value,
  };

  try {
    const response = await fetch(
      `/api/patients/${encodeURIComponent(patientId)}`,
      {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(updatedData),
      },
    );

    if (!response.ok) {
      throw new Error(await response.text());
    }

    showToast("Patient updated successfully!");
    closeModal(button);

    loadData();
  } catch (error) {
    console.error("Error updating patient:", error);
    showToast("Failed to update patient.");
  }
}

async function removePatient(patientId) {
  if (!currentUserPermissions.canDeletePatients) {
    showToast("You do not have permission to delete patients.");
    return;
  }

  if (
    !confirm(
      `Are you sure you want to permanently delete patient ${patientId}?`,
    )
  ) {
    return;
  }

  try {
    const response = await fetch(
      `/api/patients/${encodeURIComponent(patientId)}`,
      {
        method: "DELETE",
      },
    );

    if (!response.ok) {
      throw new Error(await response.text());
    }

    showToast("Patient deleted successfully.");

    loadData();
  } catch (error) {
    console.error("Error deleting patient:", error);
    showToast("Failed to delete patient.");
  }
}
