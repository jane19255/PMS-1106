const appointments = [
    { appointmentId: "APP-001", priority: "Normal", doctor: "Dr. Jimmy Jitaraphol", date: "2026-06-03", time: "9:00 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-001", firstName: "Off", lastName: "Jumpol", dob: "2000-05-12", gender: "Male", phone: "91234567", email: "off.jumpol@csc.singaporehealth.sg" } },
    { appointmentId: "APP-002", priority: "Normal", doctor: "Dr. John Lee", date: "2026-06-03", time: "10:00 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: ["wheelchair"], status: "Completed", patient: { id: "PAT-002", firstName: "Gun", lastName: "Atthaphan", dob: "1999-03-22", gender: "Female", phone: "92345678", email: "gun.atthaphan@csc.singaporehealth.sg" } },
    { appointmentId: "APP-003", priority: "Urgent", doctor: "Dr. Amelia Tan", date: "2026-06-03", time: "11:00 AM", room: "Room 3", type: "New Consultation", referringProvider: "Dr. Smith", specialRequirements: ["translator"], status: "Completed", patient: { id: "PAT-003", firstName: "Junior", lastName: "Panuwat", dob: "2002-07-10", gender: "Male", phone: "93456789", email: "junior.panuwat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-004", priority: "Emergency", doctor: "Dr. Sarah Wong", date: "2026-06-03", time: "1:00 PM", room: "Room 1", type: "Emergency", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-004", firstName: "Mark", lastName: "Siwat", dob: "2001-02-05", gender: "Female", phone: "94567890", email: "mark.siwat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-005", priority: "Normal", doctor: "Dr. John Lee", date: "2026-06-03", time: "2:00 PM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "No-Show", patient: { id: "PAT-005", firstName: "William", lastName: "Jakrapatr", dob: "2003-09-18", gender: "Male", phone: "95678901", email: "william.jakrapatr@csc.singaporehealth.sg" } },
    { appointmentId: "APP-006", priority: "Follow-up", doctor: "Dr. Amelia Tan", date: "2026-06-04", time: "9:30 AM", room: "Room 3", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-006", firstName: "Est", lastName: "Werawat", dob: "2004-11-30", gender: "Female", phone: "96789012", email: "est.werawat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-007", priority: "Normal", doctor: "Dr. Sarah Wong", date: "2026-06-04", time: "10:30 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-007", firstName: "Sea", lastName: "Tawinan", dob: "2002-04-14", gender: "Female", phone: "97890123", email: "sea.tawinan@csc.singaporehealth.sg" } },
    { appointmentId: "APP-008", priority: "Follow-up", doctor: "Dr. John Lee", date: "2026-06-04", time: "11:30 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-008", firstName: "Keng", lastName: "Harit", dob: "2001-01-09", gender: "Male", phone: "98901234", email: "keng.harit@csc.singaporehealth.sg" } },
    { appointmentId: "APP-009", priority: "Urgent", doctor: "Dr. Amelia Tan", date: "2026-06-04", time: "1:00 PM", room: "Room 3", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-009", firstName: "Namping", lastName: "Napasatkron", dob: "2003-06-07", gender: "Female", phone: "99012345", email: "namping.napasatkron@csc.singaporehealth.sg" } },
    { appointmentId: "APP-010", priority: "Emergency", doctor: "Dr. Sarah Wong", date: "2026-06-04", time: "2:00 PM", room: "Room 1", type: "Emergency", referringProvider: "", specialRequirements: ["wheelchair"], status: "Completed", patient: { id: "PAT-010", firstName: "Tle", lastName: "Thanapon", dob: "2000-12-12", gender: "Male", phone: "90123456", email: "tle.thanapon@csc.singaporehealth.sg" } },
    { appointmentId: "APP-011", priority: "Normal", doctor: "Dr. Jimmy Jitaraphol", date: "2026-06-04", time: "3:00 PM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "No-Show", patient: { id: "PAT-011", firstName: "Prem", lastName: "Warod", dob: "2002-02-02", gender: "Male", phone: "91122334", email: "prem.warod@csc.singaporehealth.sg" } },
    { appointmentId: "APP-012", priority: "Normal", doctor: "Dr. John Lee", date: "2026-06-03", time: "4:00 PM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Cancelled", patient: { id: "PAT-012", firstName: "Boss", lastName: "Teerapat", dob: "2001-08-08", gender: "Male", phone: "92233445", email: "boss.teerapat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-013", priority: "Normal", doctor: "Dr. Amelia Tan", date: "2026-06-03", time: "9:30 AM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-013", firstName: "Aun", lastName: "Napat", dob: "2003-03-03", gender: "Male", phone: "93344556", email: "aun.napat@csc.singaporehealth.sg" } },
    { appointmentId: "APP-014", priority: "Urgent", doctor: "Dr. Sarah Wong", date: "2026-06-04", time: "10:00 AM", room: "Room 1", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-014", firstName: "Lay", lastName: "Laosam", dob: "2002-09-09", gender: "Female", phone: "94455667", email: "lay.laosam@csc.singaporehealth.sg" } },
    { appointmentId: "APP-015", priority: "Normal", doctor: "Dr. Jimmy Jitaraphol", date: "2026-06-04", time: "11:00 AM", room: "Room 1", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Completed", patient: { id: "PAT-015", firstName: "Mook", lastName: "Pimchanok", dob: "2001-04-04", gender: "Female", phone: "95566778", email: "mook.pimchanok@csc.singaporehealth.sg" } },
    { appointmentId: "APP-016", priority: "Normal", doctor: "Dr. John Lee", date: "2026-06-04", time: "2:30 PM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "No-Show", patient: { id: "PAT-016", firstName: "Champ", lastName: "Krit", dob: "2000-11-11", gender: "Male", phone: "96677889", email: "champ.krit@csc.singaporehealth.sg" } },
    { appointmentId: "APP-017", priority: "Normal", doctor: "Dr. Jimmy Jitaraphol", date: "2026-06-05", time: "8:30 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Checked-In", patient: { id: "PAT-017", firstName: "Alice", lastName: "Johnson", dob: "1995-05-15", gender: "Female", phone: "97788990", email: "alice.j@csc.singaporehealth.sg" } },
    { appointmentId: "APP-018", priority: "Normal", doctor: "Dr. Richard", date: "2026-06-05", time: "9:00 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Checked-In", patient: { id: "PAT-018", firstName: "Michael", lastName: "Tan", dob: "1998-06-20", gender: "Male", phone: "98899001", email: "m.tan@csc.singaporehealth.sg" } },
    { appointmentId: "APP-019", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-05", time: "9:20 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Vitals-Done", patient: { id: "PAT-019", firstName: "Siti", lastName: "Aisyah", dob: "1999-07-25", gender: "Female", phone: "99900112", email: "siti.a@csc.singaporehealth.sg" } },
    { appointmentId: "APP-020", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-05", time: "9:40 AM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "In-Room", patient: { id: "PAT-020", firstName: "David", lastName: "Lee", dob: "2000-08-30", gender: "Male", phone: "90011223", email: "david.lee@csc.singaporehealth.sg" } },
    { appointmentId: "APP-021", priority: "Normal", doctor: "Dr. Wong", date: "2026-06-05", time: "10:00 AM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-021", firstName: "Jenny", lastName: "Low", dob: "2001-09-10", gender: "Female", phone: "91122334", email: "jenny.l@csc.singaporehealth.sg" } },
    { appointmentId: "APP-022", priority: "Normal", doctor: "Dr. Richard", date: "2026-06-05", time: "10:30 AM", room: "Room 1", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-022", firstName: "Robert", lastName: "Chen", dob: "2002-10-15", gender: "Male", phone: "92233445", email: "robert.c@csc.singaporehealth.sg" } },
    { appointmentId: "APP-023", priority: "Urgent", doctor: "Dr. Amelia Tan", date: "2026-06-05", time: "11:00 AM", room: "Room 3", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Checked-In", patient: { id: "PAT-023", firstName: "Lisa", lastName: "Ng", dob: "2003-11-20", gender: "Female", phone: "93344556", email: "lisa.ng@csc.singaporehealth.sg" } },
    { appointmentId: "APP-024", priority: "Normal", doctor: "Dr. Wong", date: "2026-06-05", time: "1:00 PM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-024", firstName: "Tom", lastName: "Smith", dob: "2000-01-05", gender: "Male", phone: "94455667", email: "tom.smith@csc.singaporehealth.sg" } },
    { appointmentId: "APP-025", priority: "Normal", doctor: "Dr. Sarah Wong", date: "2026-06-05", time: "2:00 PM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-025", firstName: "Jane", lastName: "Tan", dob: "1999-02-10", gender: "Female", phone: "95566778", email: "jane.t@csc.singaporehealth.sg" } },
    { appointmentId: "APP-026", priority: "Normal", doctor: "Dr. Jimmy Jitaraphol", date: "2026-06-05", time: "3:00 PM", room: "Room 1", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-026", firstName: "Peter", lastName: "Parker", dob: "1998-03-15", gender: "Male", phone: "96677889", email: "peter.p@csc.singaporehealth.sg" } },
    { appointmentId: "APP-027", priority: "Normal", doctor: "Dr. Richard", date: "2026-06-06", time: "9:00 AM", room: "Room 1", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-027", firstName: "Tony", lastName: "Stark", dob: "1970-04-10", gender: "Male", phone: "97788990", email: "tony.s@csc.singaporehealth.sg" } },
    { appointmentId: "APP-028", priority: "Normal", doctor: "Dr. Lee", date: "2026-06-06", time: "10:00 AM", room: "Room 2", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-028", firstName: "Steve", lastName: "Rogers", dob: "1980-05-15", gender: "Male", phone: "98899001", email: "steve.r@csc.singaporehealth.sg" } },
    { appointmentId: "APP-029", priority: "Urgent", doctor: "Dr. Wong", date: "2026-06-06", time: "11:00 AM", room: "Room 3", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-029", firstName: "Natasha", lastName: "Romanoff", dob: "1985-06-20", gender: "Female", phone: "99900112", email: "natasha.r@csc.singaporehealth.sg" } },
    { appointmentId: "APP-030", priority: "Emergency", doctor: "Dr. Sarah Wong", date: "2026-06-06", time: "1:00 PM", room: "Room 1", type: "Emergency", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-030", firstName: "Bruce", lastName: "Banner", dob: "1975-07-25", gender: "Male", phone: "90011223", email: "bruce.b@csc.singaporehealth.sg" } },
    { appointmentId: "APP-031", priority: "Normal", doctor: "Dr. John Lee", date: "2026-06-06", time: "2:00 PM", room: "Room 2", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-031", firstName: "Thor", lastName: "Odinson", dob: "1990-08-30", gender: "Male", phone: "91122334", email: "thor.o@csc.singaporehealth.sg" } },
    { appointmentId: "APP-032", priority: "Normal", doctor: "Dr. Amelia Tan", date: "2026-06-07", time: "9:30 AM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-032", firstName: "Loki", lastName: "Laufeyson", dob: "1991-09-10", gender: "Male", phone: "92233445", email: "loki.l@csc.singaporehealth.sg" } },
    { appointmentId: "APP-033", priority: "Normal", doctor: "Dr. Jimmy Jitaraphol", date: "2026-06-07", time: "10:30 AM", room: "Room 1", type: "Follow-up", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-033", firstName: "Wanda", lastName: "Maximoff", dob: "1992-10-15", gender: "Female", phone: "93344556", email: "wanda.m@csc.singaporehealth.sg" } },
    { appointmentId: "APP-034", priority: "Urgent", doctor: "Dr. John Lee", date: "2026-06-07", time: "11:30 AM", room: "Room 2", type: "New Consultation", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-034", firstName: "Vision", lastName: "Android", dob: "2010-11-20", gender: "Male", phone: "94455667", email: "vision.a@csc.singaporehealth.sg" } },
    { appointmentId: "APP-035", priority: "Normal", doctor: "Dr. Amelia Tan", date: "2026-06-07", time: "1:00 PM", room: "Room 3", type: "Routine Checkup", referringProvider: "", specialRequirements: [], status: "Scheduled", patient: { id: "PAT-035", firstName: "Carol", lastName: "Danvers", dob: "1988-12-25", gender: "Female", phone: "95566778", email: "carol.d@csc.singaporehealth.sg" } },
    { appointmentId: "APP-036", priority: "Emergency", doctor: "Dr. Sarah Wong", date: "2026-06-07", time: "2:00 PM", room: "Room 1", type: "Emergency", referringProvider: "", specialRequirements: ["wheelchair"], status: "Scheduled", patient: { id: "PAT-036", firstName: "Stephen", lastName: "Strange", dob: "1979-01-05", gender: "Male", phone: "96677889", email: "stephen.s@csc.singaporehealth.sg" } }
];

let availableDoctorNames = null;

const pagination = new Pagination({
    data: appointments,
    rowsPerPage: 3,
    tbodyId: "appointmentTableBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderAppointmentRow
});

function renderAppointmentRow(item, index) {
    const statusClass = item.status.toLowerCase().replace(/-/g, '');
    const priorityClass = item.priority.toLowerCase().replace(/-/g, '');
    return `
    <tr>
        <td>${item.appointmentId}</td>
        <td>${item.patient.firstName} ${item.patient.lastName}</td>
        <td>${item.doctor}</td>
        <td>${item.date}</td>
        <td>${item.time}</td>
        <td>${item.type}</td>
        <td><span class="status ${statusClass}">${item.status}</span></td>
        <td><span class="priority ${priorityClass}">${item.priority}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewAppointment(${index})"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            <div class="has-tooltip">
                <i class="edit fa-solid fa-pen-to-square" onclick="editAppointment(${index})"></i>
                <span class="tooltip-text">Edit Details</span>
            </div>
        </td>
    </tr>
  `;
}

function parseDate(dateStr) {
    return new Date(dateStr);
}

function formatDisplayDate(dateStr) {
    const d = new Date(dateStr);
    const day = d.getDate();
    const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const month = monthNames[d.getMonth()];
    const year = d.getFullYear();
    return `${day} ${month} ${year}`;
}

function convertToInputDate(dateStr) {
    return dateStr;
}

function filterByDateRange(list, dateRange) {
    if (!dateRange || dateRange === "") return list;

    const today = new Date();
    today.setHours(0, 0, 0, 0);

    const oneWeekLater = new Date(today);
    oneWeekLater.setDate(today.getDate() + 7);

    const thisMonth = new Date(today.getFullYear(), today.getMonth() + 1, 0);

    return list.filter(item => {
        const apptDate = parseDate(item.date);
        apptDate.setHours(0, 0, 0, 0);

        switch (dateRange) {
            case "today":
                return apptDate.getTime() === today.getTime();
            case "thisweek":
                return apptDate >= today && apptDate < oneWeekLater;
            case "thismonth":
                return apptDate.getMonth() === today.getMonth() && apptDate.getFullYear() === today.getFullYear();
            default:
                return true;
        }
    });
}

function refreshAppointmentList() {
    let list = [...appointments];

    if (availableDoctorNames !== null) {
        list = list.filter(appointment => availableDoctorNames.includes(appointment.doctor));
    }

    // Search filter
    const keyword = document.getElementById("searchInput")?.value.toLowerCase() || "";
    if (keyword) {
        list = list.filter(a =>
            a.appointmentId.toLowerCase().includes(keyword) ||
            a.patient.firstName.toLowerCase().includes(keyword) ||
            a.patient.lastName.toLowerCase().includes(keyword) ||
            a.doctor.toLowerCase().includes(keyword) ||
            a.type.toLowerCase().includes(keyword)
        );
    }

    // Status filter 
    const status = document.getElementById("filter-status")?.value;
    if (status) list = list.filter(a => a.status === status);

    // Appointment type filter
    const types = Array.from(document.querySelectorAll(".filter-type:checked")).map(c => {
        const typeMap = {
            "checkup": "Routine Checkup",
            "followup": "Follow-up",
            "consultation": "New Consultation",
            "emergency": "Emergency"
        };
        return typeMap[c.value] || c.value;
    });
    if (types.length > 0) list = list.filter(a => types.includes(a.type));

    // Doctor filter
    const doctors = Array.from(document.querySelectorAll(".filter-doctor:checked")).map(c => {
        const doctorMap = {
            "Jimmy": "Dr. Jimmy Jitaraphol",
            "amelia": "Dr. Amelia Tan",
            "john": "Dr. John Lee",
            "sarah": "Dr. Sarah Wong"
        };
        return doctorMap[c.value] || c.value;
    });
    if (doctors.length > 0) list = list.filter(a => doctors.includes(a.doctor));

    // Date range filter
    const dateRange = document.getElementById("filter-date")?.value;
    list = filterByDateRange(list, dateRange);

    // Sort logic
    const sort = document.getElementById("sortBy").value;
    list.sort((a, b) => {
        if (sort === "dateNewest") return parseDate(b.date) - parseDate(a.date);
        if (sort === "dateOldest") return parseDate(a.date) - parseDate(b.date);
        if (sort === "time") return a.time.localeCompare(b.time);
        if (sort === "status") return a.status.localeCompare(b.status);
        if (sort === "doctor") return a.doctor.localeCompare(b.doctor);
        return 0;
    });

    // Update pagination and re-render
    pagination.data = list;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function loadPatients() {
    const allPatients = appointments.map(item => ({
        name: `${item.patient.firstName} ${item.patient.lastName}`,
        id: item.patient.id
    }));

    const uniquePatients = Array.from(new Map(allPatients.map(p => [p.name, p])).values());

    const addSelect = document.getElementById("patientsList");
    if (addSelect) {
        uniquePatients.forEach(p => {
            addSelect.innerHTML += `<option value="${p.name}">${p.name}</option>`;
        });
    }

    const editSelect = document.getElementById("editPatientsList");
    if (editSelect) {
        uniquePatients.forEach(p => {
            editSelect.innerHTML += `<option value="${p.name}">${p.name}</option>`;
        });
    }
}

async function loadDoctors() {
    let doctors;

    try {
        const response = await fetch("/api/doctors");
        if (!response.ok) throw new Error(`Doctor API returned ${response.status}`);

        doctors = await response.json();
    } catch (error) {
        console.warn("Could not load doctors from backend:", error);
        doctors = appointments.map(item => ({ name: item.doctor }));
    }

    const uniqueDoctors = Array.from(
        new Map(doctors.map(doctor => [doctor.name, doctor])).values()
    ).filter(doctor => doctor.name);
    availableDoctorNames = uniqueDoctors.map(doctor => doctor.name);

    const addSelect = document.getElementById("doctorList");
    if (addSelect) {
        addSelect.innerHTML = "";
        uniqueDoctors.forEach(doctor => {
            addSelect.innerHTML += `<option value="${doctor.name}">${doctor.name}</option>`;
        });
    }

    const editSelect = document.getElementById("editDoctorList");
    if (editSelect) {
        editSelect.innerHTML = "";
        uniqueDoctors.forEach(doctor => {
            editSelect.innerHTML += `<option value="${doctor.name}">${doctor.name}</option>`;
        });
    }

    const checkbox = document.getElementById("doctorCheckbox");
    if (checkbox) {
        checkbox.innerHTML = "";
        uniqueDoctors.forEach(doctor => {
            checkbox.innerHTML += `<div><input type="checkbox" class="filter-doctor" id="${doctor.name}" name="${doctor.name}" value="${doctor.name}">
                                        <label for="${doctor.name}">${doctor.name}</label>
                                    </div>`;
        });
    }
}

function viewAppointment(index) {
    currentEditIndex = null;
    openModal("detailsModal");
    fillViewData(appointments[index]);
}

function editAppointment(index) {
    currentEditIndex = index;
    openModal("editModal");
    fillEditData(appointments[index]);
}

function fillViewData(item) {
    document.getElementById("view-doc").innerText = item.doctor;
    document.getElementById("view-date").innerText = item.date;
    document.getElementById("view-time").innerText = item.time;
    document.getElementById("view-type").innerText = item.type;
    document.getElementById("view-room").innerText = item.room;
    document.getElementById("view-patient").innerText = `${item.patient.firstName} ${item.patient.lastName}`;
    document.getElementById("view-sr").innerText = item.specialRequirements.length > 0
        ? item.specialRequirements.join(", ")
        : "None";
    document.getElementById("view-rp").innerText = item.referringProvider || "None";
}

function fillEditData(item) {
    document.getElementById("editDoctorList").value = item.doctor;
    document.getElementById("edit_appointment_date").value = convertToInputDate(item.date);
    document.getElementById("editAppointmentRoom").value = item.room;
    document.getElementById("appointmentType").value = item.type;
    document.getElementById("editPatientsList").value = `${item.patient.firstName} ${item.patient.lastName}`;
    document.getElementById("editWheelchair").checked = item.specialRequirements.includes("wheelchair");
    document.getElementById("editEquipment").checked = item.specialRequirements.includes("medical equipment");
    document.getElementById("editTranslator").checked = item.specialRequirements.includes("translator");
    document.querySelector("#editModal textarea[placeholder='Name of external doctor']").value = item.referringProvider;
    const priorityLevels = document.querySelectorAll("#editModal .level div");
    priorityLevels.forEach(level => {
        level.classList.remove("selected");
        if (level.textContent.trim().toLowerCase() === item.priority.toLowerCase()) {
            level.classList.add("selected");
        }
    });
}

function showAddPatientTab(cb) {
    const tab = document.getElementById("addNewPatientTab");
    const inputs = tab.querySelectorAll('input, textarea, select');
    const labels = tab.querySelectorAll("label");
    tab.classList.toggle('show');

    inputs.forEach(input => {
        if (!cb.checked) {
            if (input.type === 'checkbox' || input.type === 'radio') {
                input.checked = false;
            } else {
                input.value = '';
            }
            input.required = false;
        } else {
            input.required = true;
        }
    });
};

function goToStep(modal, targetStep) {
    const maxSteps = 3;

    modal.querySelectorAll('.step-content').forEach(content => {
        content.classList.remove('active');
    });
    modal.querySelectorAll('.step-item').forEach(item => {
        item.classList.remove('active');
    });

    modal.querySelectorAll('.step-item').forEach(item => {
        const stepNum = parseInt(item.dataset.step);
        if (stepNum < targetStep) {
            item.classList.add('completed');
        }
    });

    modal.querySelector(`.step-content[data-step="${targetStep}"]`).classList.add('active');
    modal.querySelector(`.step-item[data-step="${targetStep}"]`).classList.add('active');

    const backBtn = modal.querySelector('.btn-back');
    backBtn.style.display = targetStep === 1 ? 'none' : 'block';

    const nextBtn = modal.querySelector('.btn-next');
    const modalId = modal.id;
    if (targetStep === maxSteps) {
        nextBtn.innerHTML = '<i class="fa-solid fa-floppy-disk"></i>Submit';
        if (modalId === 'editModal') {
            nextBtn.setAttribute("onclick", "showToast('Changes saved!', 'success')&&backToFirstStep(this)&&closeModal(this)");
        } else {
            nextBtn.setAttribute("onclick", "showToast('Craeted new appointment!', 'success')&&backToFirstStep(this)&&closeModal(this)&&clearInput(this)");
        }
    }
    else {
        nextBtn.textContent = "Next";
        if (modalId === 'editModal') {
            nextBtn.setAttribute("onclick", `verifyInput('#editModal .step-content[data-step="${targetStep}"]')&&nextStep(this)`);
        } else {
            nextBtn.setAttribute("onclick", `verifyInput('#addAppointmentModal .step-content[data-step="${targetStep}"]')&&nextStep(this)`);
        }
    }
}

function nextStep(button) {
    const modal = button.closest('.modal');
    const currentStep = parseInt(modal.querySelector('.step-item.active').dataset.step);
    const maxSteps = 3;

    if (currentStep < maxSteps) {
        goToStep(modal, currentStep + 1);
    }
}

function prevStep(button) {
    const modal = button.closest('.modal');
    const currentStep = parseInt(modal.querySelector('.step-item.active').dataset.step);

    if (currentStep > 1) {
        goToStep(modal, currentStep - 1);
    }
}

function backToFirstStep(button) {
    const modal = button.closest('.modal');
    const backBtn = modal.querySelector('.btn-back');
    backBtn.click();
    backBtn.click();

    const steps = modal.querySelectorAll(".step-item");
    steps.forEach(step => {
        step.classList.remove("completed")
    });

    return true;
}

document.addEventListener("DOMContentLoaded", async () => {
    loadPatients();
    await loadDoctors();
    refreshAppointmentList();

    flatpickr("#appointment_date", {
        enableTime: false,
        dateFormat: "Y-m-d",
        minDate: "today",
        allowInput: false,
        theme: "light",
    });

    flatpickr("#edit_appointment_date", {
        enableTime: false,
        dateFormat: "Y-m-d",
        minDate: "today",
        allowInput: false,
        theme: "light",
    });

    const slots = document.querySelectorAll(".timeslot .slot");
    slots.forEach(slot => {
        slot.addEventListener('click', (event) => {
            if (!slot.classList.contains('disable')) {
                slot.classList.toggle("selected")
            }
        });
    });

    const levels = document.querySelectorAll(".level div");
    levels.forEach(level => {
        level.addEventListener('click', (event) => {
            levels.forEach(el => el.classList.remove("selected"));
            level.classList.add("selected");
        });
    });
});
