const appointments = [
    { appointmentId: "APP-001", priority: "Normal", doctor: "Dr. Richard", date: "2026-06-03", time: "9:00 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-001", firstName: "Off", lastName: "Jumpol", dob: "2000-05-12", gender: "Male", phone: "91234567", email: "off.jumpol@csc.singaporehealth.sg" } },
    { appointmentId: "APP-002", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-03", time: "10:00 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: ["wheelchair"], status: "Completed", patient: { id: "PAT-002", firstName: "Gun", lastName: "Atthaphan", dob: "1999-03-22", gender: "Female", phone: "92345678", email: "gun.atthaphan@csc.singaporehealth.sg" } },
    { appointmentId: "APP-003", priority: "Urgent", doctor: "Dr. Wong", date: "2026-06-03", time: "11:00 AM", room: "Room 3", type: "New Consultation", referringProvider: "Dr. Smith", specialRequirements: ["translator"], status: "Completed", patient: { id: "PAT-003", firstName: "Junior", lastName: "Panuwat", dob: "2002-07-10", gender: "Male", phone: "93456789", email: "junior.panuwat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-004", priority: "Emergency", doctor: "Dr. Richard", date: "2026-06-03", time: "1:00 PM", room: "Room 1", type: "Emergency", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-004", firstName: "Mark", lastName: "Siwat", dob: "2001-02-05", gender: "Female", phone: "94567890", email: "mark.siwat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-005", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-03", time: "2:00 PM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "No-Show", patient: { id: "PAT-005", firstName: "William", lastName: "Jakrapatr", dob: "2003-09-18", gender: "Male", phone: "95678901", email: "william.jakrapatr@csc.singaporehealth.sg" } },
    { appointmentId: "APP-006", priority: "Follow-up", doctor: "Dr. Richard", date: "2026-06-04", time: "9:30 AM", room: "Room 1", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-006", firstName: "Est", lastName: "Werawat", dob: "2004-11-30", gender: "Female", phone: "96789012", email: "est.werawat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-007", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-04", time: "10:30 AM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-007", firstName: "Sea", lastName: "Tawinan", dob: "2002-04-14", gender: "Female", phone: "97890123", email: "sea.tawinan@csc.singaporehealth.sg" } },
    { appointmentId: "APP-008", priority: "Follow-up", doctor: "Dr. Wong", date: "2026-06-04", time: "11:30 AM", room: "Room 3", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-008", firstName: "Keng", lastName: "Harit", dob: "2001-01-09", gender: "Male", phone: "98901234", email: "keng.harit@csc.singaporehealth.sg" } },
    { appointmentId: "APP-009", priority: "Urgent", doctor: "Dr. Richard", date: "2026-06-04", time: "1:00 PM", room: "Room 1", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-009", firstName: "Namping", lastName: "Napasatkron", dob: "2003-06-07", gender: "Female", phone: "99012345", email: "namping.napasatkron@csc.singaporehealth.sg" } },
    { appointmentId: "APP-010", priority: "Emergency", doctor: "Dr. Lee", date: "2026-06-04", time: "2:00 PM", room: "Room 2", type: "Emergency", referringProvider: "", specialRequirements: ["wheelchair"], status: "Completed", patient: { id: "PAT-010", firstName: "Tle", lastName: "Thanapon", dob: "2000-12-12", gender: "Male", phone: "90123456", email: "tle.thanapon@csc.singaporehealth.sg" } },
    { appointmentId: "APP-011", priority: "Normal", doctor: "Dr. Wong", date: "2026-06-04", time: "3:00 PM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "No-Show", patient: { id: "PAT-011", firstName: "Prem", lastName: "Warod", dob: "2002-02-02", gender: "Male", phone: "91122334", email: "prem.warod@csc.singaporehealth.sg" } },
    { appointmentId: "APP-017", priority: "Normal", doctor: "Dr. Richard", date: getTodayDate(), time: "8:30 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Checked-In", patient: { id: "PAT-017", firstName: "Alice", lastName: "Johnson", dob: "1995-05-15", gender: "Female", phone: "97788990", email: "alice.j@csc.singaporehealth.sg" } },
    { appointmentId: "APP-018", priority: "Normal", doctor: "Dr. Richard", date: getTodayDate(), time: "9:00 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Checked-In", patient: { id: "PAT-018", firstName: "Michael", lastName: "Tan", dob: "1998-06-20", gender: "Male", phone: "98899001", email: "m.tan@csc.singaporehealth.sg" } },
    { appointmentId: "APP-019", priority: "Normal", doctor: "Dr. Lee", date: getTodayDate(), time: "9:20 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Vitals-Done", patient: { id: "PAT-019", firstName: "Siti", lastName: "Aisyah", dob: "1999-07-25", gender: "Female", phone: "99900112", email: "siti.a@csc.singaporehealth.sg" } },
    { appointmentId: "APP-020", priority: "Normal", doctor: "Dr. Lee", date: getTodayDate(), time: "9:40 AM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "In-Room", patient: { id: "PAT-020", firstName: "David", lastName: "Lee", dob: "2000-08-30", gender: "Male", phone: "90011223", email: "david.lee@csc.singaporehealth.sg" } },
    { appointmentId: "APP-021", priority: "Normal", doctor: "Dr. Wong", date: getTodayDate(), time: "10:00 AM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-021", firstName: "Jenny", lastName: "Low", dob: "2001-09-10", gender: "Female", phone: "91122334", email: "jenny.l@csc.singaporehealth.sg" } },
    { appointmentId: "APP-022", priority: "Normal", doctor: "Dr. Richard", date: getTodayDate(), time: "10:30 AM", room: "Room 1", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-022", firstName: "Robert", lastName: "Chen", dob: "2002-10-15", gender: "Male", phone: "92233445", email: "robert.c@csc.singaporehealth.sg" } },
    { appointmentId: "APP-023", priority: "Urgent", doctor: "Dr. Wong", date: getTodayDate(), time: "11:00 AM", room: "Room 3", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Checked-In", patient: { id: "PAT-023", firstName: "Lisa", lastName: "Ng", dob: "2003-11-20", gender: "Female", phone: "93344556", email: "lisa.ng@csc.singaporehealth.sg" } },
    { appointmentId: "APP-027", priority: "Normal", doctor: "Dr. Richard", date: "2026-06-06", time: "9:00 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-027", firstName: "Tony", lastName: "Stark", dob: "1970-04-10", gender: "Male", phone: "97788990", email: "tony.s@csc.singaporehealth.sg" } },
    { appointmentId: "APP-028", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-06", time: "10:00 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-028", firstName: "Steve", lastName: "Rogers", dob: "1980-05-15", gender: "Male", phone: "98899001", email: "steve.r@csc.singaporehealth.sg" } },
    { appointmentId: "APP-029", priority: "Urgent", doctor: "Dr. Wong", date: "2026-06-06", time: "11:00 AM", room: "Room 3", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-029", firstName: "Natasha", lastName: "Romanoff", dob: "1985-06-20", gender: "Female", phone: "99900112", email: "natasha.r@csc.singaporehealth.sg" } },
];

const pagination = new Pagination({
    data: appointments,
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
    const todaysApps = appointments.filter(a => a.date === today);
    return [...new Set(todaysApps.map(a => a.doctor))];
}

function getRoomStatuses() {
    const today = getTodayDate();
    const doctors = getTodaysDoctors();
    const roomList = ["Room 1", "Room 2", "Room 3"];
    const rooms = doctors.map((doc, i) => ({ doctor: doc, roomNumber: roomList[i] }));
    while (rooms.length < 3) rooms.push({ doctor: "Unassigned", roomNumber: roomList[rooms.length] });

    return rooms.map(room => {
        const todaysApps = appointments.filter(a => a.date === today && a.doctor === room.doctor).sort((a, b) => a.time.localeCompare(b.time));
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
    const filtered = appointments.filter(p => p.date === selectedDate);

    // Update pagination data
    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function findAppointmentByName(fullName) {
    return appointments.find(a => `${a.patient.firstName} ${a.patient.lastName}` === fullName);
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

    let filtered = appointments.filter(item => item.date === currentSelectedDate);

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
}

function initFilters() {
    const search = document.getElementById("searchInput");
    const doctor = document.getElementById("doctorFilter");
    const status = document.getElementById("statusFilter");

    search.addEventListener("input", applyAllFiltersAndRefresh);
    doctor.addEventListener("change", applyAllFiltersAndRefresh);
    status.addEventListener("change", applyAllFiltersAndRefresh);
}


function loadDoctors() {
    const allDoctors = appointments.map(item => ({
        name: `${item.doctor}`
    }));

    const uniqueDoctors = Array.from(new Map(allDoctors.map(d => [d.name, d])).values());

    const doctorFilter = document.getElementById("doctorFilter");
    if (doctorFilter) {
        uniqueDoctors.forEach(p => {
            doctorFilter.innerHTML += `<option value="${p.name}">${p.name}</option>`;
        });
    }
}

document.addEventListener("DOMContentLoaded", () => {
    loadDoctors();
    renderRoomTable();
    initFilters();
    flatpickr("#fullCalendar", {
        inline: true, dateFormat: "Y-m-d", defaultDate: getTodayDate(),
        onChange: (_, dateStr) => refreshAppointmentTable(dateStr)
    });
    refreshAppointmentTable(getTodayDate());
});
