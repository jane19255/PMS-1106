let appointments = [];

let activeDoctorNames = null;
let currentSelectedDate = getTodayDate();

function getDashboardAppointments() {
    if (activeDoctorNames === null) return appointments;
    return appointments.filter(appointment => activeDoctorNames.includes(appointment.doctor));
}

const pagination = new Pagination({
    data: getDashboardAppointments(),
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

function getTodaysDoctors() {
    const today = getTodayDate();
    if (activeDoctorNames !== null) return activeDoctorNames;

    const todaysApps = getDashboardAppointments().filter(a => a.date === today);
    return [...new Set(todaysApps.map(a => a.doctor))];
}

function getRoomStatuses() {
    const today = getTodayDate();
    const doctors = getTodaysDoctors();
    const roomList = ["Room 1", "Room 2", "Room 3"];
    const rooms = doctors.map((doc, i) => ({ doctor: doc, roomNumber: roomList[i] }));
    while (rooms.length < 3) rooms.push({ doctor: "Unassigned", roomNumber: roomList[rooms.length] });

    return rooms.map(room => {
        const todaysApps = getDashboardAppointments().filter(a => a.date === today && a.doctor === room.doctor).sort((a, b) => a.time.localeCompare(b.time));
        const inRoomPatient = todaysApps.find(a => a.status === "In-Room");
        const nextApp = todaysApps.find(a => ["Scheduled", "Checked-In", "Vitals-Done"].includes(a.status));
        return {
            doctor: room.doctor, roomNumber: room.roomNumber, isVacant: !inRoomPatient,
            currentPatient: inRoomPatient ? `${inRoomPatient.patient.firstName} ${inRoomPatient.patient.lastName}` : "None",
            nextAppointment: nextApp ? `${nextApp.time} | ${nextApp.patient.firstName} ${nextApp.patient.lastName}` : "No upcoming",
            freeGap: !inRoomPatient ? "Free slot available for Early/Walk-In" : "No available free time"
        };
    });
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

function renderRoomTable() {
    const roomTableBody = document.querySelector("#roomMonitorTbl tbody");
    roomTableBody.innerHTML = "";
    const rooms = getRoomStatuses();

    rooms.forEach(room => {
        const row = document.createElement("tr");
        row.dataset.doc = room.doctor;
        row.dataset.roomvacant = room.isVacant;

        row.innerHTML = `
            <td>${room.doctor}</td>
            <td>${room.roomNumber}</td>
            <td><span class="${room.isVacant ? 'tag-vacant' : 'tag-occupied'}">${room.isVacant ? 'VACANT' : 'OCCUPIED'}</span></td>
            <td>${room.currentPatient}</td>
            <td>${room.nextAppointment}</td>
            <td>${room.freeGap}</td>
        `;
        roomTableBody.appendChild(row);
    });
}

function renderAppointmentRow(app, index) {
    const status = getPatientStatus(app);
    const fullName = `${app.patient.firstName} ${app.patient.lastName}`;
    const isToday = app.date === getTodayDate();
    const isArrived = ["Checked-In", "Vitals-Done", "In-Room"].includes(app.status);
    const canEnterVitals = app.status === "Checked-In";
    const canSendRoom = app.status === "Vitals-Done";

    let html = `
    <tr data-doctor="${app.doctor}" data-status="${app.status}">
        <td>${fullName}</td>
        <td>${app.doctor}</td>
        <td>${app.time}</td>`;

    if (isToday) {
        html += `
        <td><span class="${status.class}">${status.text}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <button class="btn btn-arrive ${isArrived ? 'disabled' : ''}" ${isArrived ? 'disabled' : ''} onclick="confirmArrive('${fullName}')"><i class="fa-solid fa-user-check"></i></button>
                <span class="tooltip-text">Arrive</span>
            </div>
            <div class="has-tooltip">
                <button class="btn btn-vitals ${!canEnterVitals ? 'disabled' : ''}" ${!canEnterVitals ? 'disabled' : ''} onclick="openVital('${fullName}')"><i class="fa-solid fa-file-waveform"></i></button>
                <span class="tooltip-text">Enter Vitals</span>
            </div>
            <div class="has-tooltip">
                <button class="btn btn-sendRoom ${!canSendRoom ? 'disabled' : ''}" ${!canSendRoom ? 'disabled' : ''} onclick="sendToRoom('${fullName}')"><i class="fa-solid fa-arrow-right-from-bracket"></i></button>
                <span class="tooltip-text">Send to room</span>
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
    const filtered = getDashboardAppointments().filter(p => p.date === selectedDate);

    // Update pagination data
    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();
    updateDashboardMetrics();
}

function findAppointmentByName(fullName) {
    return getDashboardAppointments().find(a => `${a.patient.firstName} ${a.patient.lastName}` === fullName);
}

function confirmArrive(patientName) {
    const app = findAppointmentByName(patientName);
    if (app) app.status = "Checked-In";
    refreshAppointmentTable(getTodayDate());
    renderRoomTable();
}

function openVital(name) {
    document.getElementById("currentPatientName").textContent = name;
    const app = findAppointmentByName(name);
    if (app?.vitals) {
        document.getElementById("bp").value = app.vitals.bp || "";
        document.getElementById("temp").value = app.vitals.temp || "";
        document.getElementById("pulse").value = app.vitals.pulse || "";
        document.getElementById("height").value = app.vitals.height || "";
        document.getElementById("weight").value = app.vitals.weight || "";
    }
    openModal('vitalModal');
}

function saveVital() {
    const pName = document.getElementById("currentPatientName").textContent;
    const app = findAppointmentByName(pName);
    if (app) {
        app.status = "Vitals-Done";
        app.vitals = {
            bp: document.getElementById("bp").value,
            temp: document.getElementById("temp").value,
            pulse: document.getElementById("pulse").value,
            height: document.getElementById("height").value,
            weight: document.getElementById("weight").value
        };
        showToast("Vitals saved!", "success");
    }
    refreshAppointmentTable(getTodayDate());
    renderRoomTable();
    return true;
}

function sendToRoom(patientName) {
    const app = findAppointmentByName(patientName);
    if (!app) return;

    const rooms = getRoomStatuses();
    const room = rooms.find(r => r.doctor === app.doctor);
    if (!room || !room.isVacant) {
        showToast("Room is occupied!", "danger");
        return;
    }

    app.status = "In-Room";
    showToast(`${patientName} sent to room!`, "success");
    refreshAppointmentTable(getTodayDate());
    renderRoomTable();
}

function applyAllFiltersAndRefresh() {
    const searchKeyword = document.getElementById("searchInput").value.toLowerCase().trim();
    const selectedDoctor = document.getElementById("doctorFilter").value;
    const selectedStatus = document.getElementById("statusFilter").value;

    let filtered = getDashboardAppointments().filter(item => item.date === currentSelectedDate);

    // Search by patient name / doctor name
    if (searchKeyword) {
        filtered = filtered.filter(item => {
            const fullName = `${item.patient.firstName} ${item.patient.lastName}`.toLowerCase();
            const doctorName = item.doctor.toLowerCase();
            return fullName.includes(searchKeyword) || doctorName.includes(searchKeyword);
        });
    }

    // Doctor filter
    if (selectedDoctor !== "all") {
        filtered = filtered.filter(item => item.doctor === selectedDoctor);
    }

    // Status filter
    if (selectedStatus !== "all") {
        filtered = filtered.filter(item => item.status === selectedStatus);
    }

    // Update pagination
    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();
    updateDashboardMetrics();
}

function initFilters() {
    const search = document.getElementById("searchInput");
    const doctor = document.getElementById("doctorFilter");
    const status = document.getElementById("statusFilter");

    search.addEventListener("input", applyAllFiltersAndRefresh);
    doctor.addEventListener("change", applyAllFiltersAndRefresh);
    status.addEventListener("change", applyAllFiltersAndRefresh);
}


async function loadDashboardAppointments() {
    try {
        const response = await fetch("/api/doctor-dashboard/appointments", {
            headers: { Accept: "application/json" },
        });
        if (!response.ok) throw new Error(`Dashboard API returned ${response.status}`);
        appointments = await response.json();
    } catch (error) {
        console.warn("Could not load dashboard appointments:", error);
        appointments = [];
        if (typeof showToast === "function") {
            showToast("Dashboard appointments could not be loaded", "error");
        }
    }
}

function setDashboardMetric(cardId, value) {
    const number = document.querySelector(`#${cardId} .number`);
    if (number) number.textContent = value;
}

function updateDashboardMetrics() {
    const today = getTodayDate();
    const todaysAppointments = getDashboardAppointments().filter(app => app.date === today);
    const arrivedStatuses = ["Checked-In", "Vitals-Done", "In-Room", "Completed"];

    setDashboardMetric("todayAppointments", todaysAppointments.length);
    setDashboardMetric("patientArrived", todaysAppointments.filter(app => arrivedStatuses.includes(app.status)).length);
    setDashboardMetric("pendingViral", todaysAppointments.filter(app => app.status === "Checked-In").length);
    setDashboardMetric("noshow", todaysAppointments.filter(app => app.status === "No-Show").length);
}
async function loadDoctors() {
    const doctorFilter = document.getElementById("doctorFilter");

    try {
        const response = await fetch("/api/doctors");
        if (!response.ok) throw new Error(`Doctor API returned ${response.status}`);

        const doctors = await response.json();
        activeDoctorNames = doctors.map(doctor => doctor.name).filter(Boolean);
    } catch (error) {
        console.warn("Could not load doctors from backend:", error);
        activeDoctorNames = [];
    }

    if (doctorFilter) {
        doctorFilter.innerHTML = `<option value="all">All Doctors</option>`;
        activeDoctorNames.forEach(name => {
            doctorFilter.innerHTML += `<option value="${name}">${name}</option>`;
        });
    }

    pagination.data = getDashboardAppointments().filter(item => item.date === currentSelectedDate);
    pagination.currentPage = 1;
    pagination.renderTable();
    updateDashboardMetrics();
}

document.addEventListener("DOMContentLoaded", async () => {
    await loadDoctors();
    await loadDashboardAppointments();
    updateDashboardMetrics();
    renderRoomTable();
    initFilters();
    flatpickr("#fullCalendar", {
        inline: true, dateFormat: "Y-m-d", defaultDate: getTodayDate(),
        onChange: (_, dateStr) => refreshAppointmentTable(dateStr)
    });
    refreshAppointmentTable(getTodayDate());
});

