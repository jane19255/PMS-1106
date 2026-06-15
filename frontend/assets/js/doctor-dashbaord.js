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
    const filtered = appointments.filter(p => p.date === selectedDate);

    // Update pagination data
    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function findAppointmentByName(fullName) {
    return appointments.find(a => `${a.patient.firstName} ${a.patient.lastName}` === fullName);
}

function applyAllFiltersAndRefresh() {
    const searchKeyword = document.getElementById("searchInput").value.toLowerCase().trim();
    const selectedStatus = document.getElementById("statusFilter").value;
    const selectedPriority = document.getElementById("priorityFilter").value;

    let filtered = appointments.filter(item => item.date === currentSelectedDate);

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


document.addEventListener("DOMContentLoaded", () => {
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
