let appointments = [];

let activeDoctorNames = null;
let currentSelectedDate = getTodayDate();

function getActiveAppointments() {
    if (activeDoctorNames === null) return appointments;
    return appointments.filter(appointment => activeDoctorNames.includes(appointment.doctor));
}

const pagination = new Pagination({
    data: getActiveAppointments(),
    rowsPerPage: 3,
    tbodyId: "appBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderAppointmentRow
});

function getTodayDate() {
    const today = new Date();
    const year = today.getFullYear();
    const month = String(today.getMonth() + 1).padStart(2, '0');
    const day = String(today.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
}

function getPatientStatus(app) {
    switch (app.status) {
        case "Scheduled": return { class: "badge badge-notarrive", text: "Not Arrived" };
        case "Checked-In": return { class: "badge badge-pendingvital", text: "Pending Vitals" };
        case "Vitals-Done": return { class: "badge badge-ready", text: "Ready for Doctor" };
        case "In-Room": return { class: "badge badge-inroom", text: "In Consultation Room" };
        case "Completed": return { class: "badge badge-completed", text: "Completed" };
        case "No-Show": return { class: "badge badge-noshow", text: "No Show" };
        case "Cancelled": return { class: "badge badge-cancelled", text: "Cancelled" };
        default: return { class: "badge", text: app.status };
    }
}

function renderAppointmentRow(app, index) {
    const status = getPatientStatus(app);
    const fullName = `${app.patient.firstName} ${app.patient.lastName}`;
    const isToday = app.date === getTodayDate();
    const isArrived = ["Checked-In", "Vitals-Done", "In-Room"].includes(app.status);
    const canEnterVitals = app.status === "Checked-In";
    const canSendRoom = app.status === "Vitals-Done";

    let html = `
    <tr data-status="${app.status}">
        <td>${fullName}</td>
        <td>${app.time}</td>
        <td><span class="priority ${app.priority}">${app.priority}</span></td>`;

    if (isToday) {
        html += `
        <td><span class="${status.class}">${status.text}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <button id='callEarlyBtn' onclick='event.stopPropagation();showToast("Informed receptionist!", "success")'><i class="fa-regular fa-bell"></i>Call Early</button>
                <span class="tooltip-text">Call patient earlier</span>
            </div>
        </td>`;
    } else {
        html += `<td><span class="${status.class}">${status.text}</span></td><td class="action"></td>`;
    }

    html += `</tr>`;
    return html;
}

function refreshAppointmentTable(selectedDate) {
    currentSelectedDate = selectedDate;
    const filtered = getActiveAppointments().filter(p => p.date === selectedDate);

    // Update pagination data
    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function findAppointmentByName(fullName) {
    return getActiveAppointments().find(a => `${a.patient.firstName} ${a.patient.lastName}` === fullName);
}

function applyAllFiltersAndRefresh() {
    const searchKeyword = document.getElementById("searchInput").value.toLowerCase().trim();
    const selectedStatus = document.getElementById("statusFilter").value;
    const selectedPriority = document.getElementById("priorityFilter").value;

    let filtered = getActiveAppointments().filter(item => item.date === currentSelectedDate);

    // Search by patient name / doctor name
    if (searchKeyword) {
        filtered = filtered.filter(item => {
            const fullName = `${item.patient.firstName} ${item.patient.lastName}`.toLowerCase();
            return fullName.includes(searchKeyword);
        });
    }

    // Status filter
    if (selectedStatus !== "all") {
        filtered = filtered.filter(item => item.status === selectedStatus);
    }

    if (selectedPriority !== "all") {
        filtered = filtered.filter(item => item.priority === selectedPriority);
    }

    // Update pagination
    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function initFilters() {
    const search = document.getElementById("searchInput");
    const status = document.getElementById("statusFilter");
    const priority = document.getElementById("priorityFilter");

    search.addEventListener("input", applyAllFiltersAndRefresh);
    status.addEventListener("change", applyAllFiltersAndRefresh);
    priority.addEventListener("change", applyAllFiltersAndRefresh);
}


async function loadDashboardAppointments() {
    try {
        const response = await fetch("/api/doctor-dashboard/appointments", {
            headers: { Accept: "application/json" },
        });
        if (!response.ok) throw new Error(`Dashboard API returned ${response.status}`);
        appointments = await response.json();
    } catch (error) {
        console.warn("Could not load doctor dashboard appointments:", error);
        appointments = [];
        if (typeof showToast === "function") {
            showToast("Doctor appointments could not be loaded", "error");
        }
    }
}
async function loadDoctors() {
    try {
        const response = await fetch("/api/doctors");
        if (!response.ok) throw new Error(`Doctor API returned ${response.status}`);

        const doctors = await response.json();
        activeDoctorNames = doctors.map(doctor => doctor.name).filter(Boolean);
    } catch (error) {
        console.warn("Could not load doctors from backend:", error);
        activeDoctorNames = null;
    }
}

document.addEventListener("DOMContentLoaded", async () => {
    await loadDoctors();
    await loadDashboardAppointments();
    initFilters();
    flatpickr("#fullCalendar", {
        inline: true, dateFormat: "Y-m-d", defaultDate: getTodayDate(),
        onChange: (_, dateStr) => refreshAppointmentTable(dateStr)
    });
    refreshAppointmentTable(getTodayDate());

    const trs = document.querySelectorAll(".table-list tr");

    trs.forEach(tr => {
        tr.addEventListener("click", () => {
            window.location.href = "Medical-Records.html";
        });
    });
});
