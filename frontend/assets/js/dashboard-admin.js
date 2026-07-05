let currentSelectedDate = "";
let globalAppointments = [];
let globalRooms = [];

async function readApiResponse(res) {
    const text = await res.text();

    try {
        return text ? JSON.parse(text) : null;
    } catch {
        if (!res.ok) {
            throw new Error(text || "Request failed");
        }
        return text;
    }
}

const pagination = new Pagination({
    data: [],
    rowsPerPage: 3,
    tbodyId: "appBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderAppointmentRow
});

function getTodayDate() {
    return new Intl.DateTimeFormat("en-CA", {
        timeZone: "Asia/Singapore",
        year: "numeric",
        month: "2-digit",
        day: "2-digit"
    }).format(new Date());
}

function formatIsoTime(isoStr) {
    return new Intl.DateTimeFormat("en-SG", {
        timeZone: "Asia/Singapore",
        hour: "numeric",
        minute: "2-digit",
        hour12: true
    }).format(new Date(isoStr));
}

function extractIsoDate(isoStr) {
    return new Intl.DateTimeFormat("en-CA", {
        timeZone: "Asia/Singapore",
        year: "numeric",
        month: "2-digit",
        day: "2-digit"
    }).format(new Date(isoStr));
}

function updateStatsCards() {
    const todayAppointments = globalAppointments.length;

    const patientArrived = globalAppointments.filter(app =>
        ["Checked In", "Vitals Recorded", "In Consultation", "Completed"].includes(app.status)
    ).length;

    const pendingVitals = globalAppointments.filter(app =>
        app.status === "Checked In"
    ).length;

    const potentialNoShow = globalAppointments.filter(app =>
        app.status === "Scheduled"
    ).length;

    document.querySelector("#todayAppointments .number").textContent = todayAppointments;
    document.querySelector("#patientArrived .number").textContent = patientArrived;
    document.querySelector("#pendingViral .number").textContent = pendingVitals;
    document.querySelector("#noshow .number").textContent = potentialNoShow;
}

async function fetchRooms() {
    try {
        const res = await fetch("/api/dashboard/rooms");
        if (!res.ok) throw new Error("Failed to load rooms");

        const rawRooms = await res.json();

        // Offboarded doctors keep their room row (for history), and a doctor
        // marked Unavailable/On Leave still keeps theirs too — neither should
        // show up as an active room on the monitor.
        globalRooms = rawRooms
            .filter(row => (row.doctors?.staff?.status || "active").toLowerCase() !== "inactive")
            .filter(row => (row.doctors?.availability_status || "Available") === "Available")
            .map(row => ({
                doctorId: row.doctor_id,
                doctor: row.doctors?.staff?.full_name || "Unknown Doctor",
                roomNumber: row.room || "Unassigned",
                roomStatus: row.status || "Unavailable",
                currentAppointmentId: row.current_appointment_id || null
            }));

        return globalRooms;
    } catch (err) {
        console.error(err);
        showToast("Failed to load room data", "danger");
        globalRooms = [];
        return [];
    }
}

async function loadDoctors() {
    const doctorFilter = document.getElementById("doctorFilter");
    doctorFilter.innerHTML = `<option value="all">All Doctors</option>`;
    const todayStr = getTodayDate();
    await refreshAppointmentTable(todayStr);
    const uniqueDoctors = [...new Set(globalAppointments.map(i => i.doctor))];
    uniqueDoctors.forEach(name => {
        doctorFilter.innerHTML += `<option value="${name}">${name}</option>`;
    });
}

function getRoomStatuses() {
    const todaysApps = globalAppointments.filter(a => a.date === currentSelectedDate);

    return globalRooms.map(room => {
        const roomApps = todaysApps
            .filter(a => a.doctorId === room.doctorId)
            .sort((a, b) => new Date(a.raw.scheduled_at) - new Date(b.raw.scheduled_at));

        const currentApp =
            roomApps.find(a => a.appointmentId === room.currentAppointmentId) ||
            roomApps.find(a => a.status === "In Consultation");

        const nextApp = roomApps.find(a =>
            ["Scheduled", "Checked In", "Vitals Recorded"].includes(a.status)
        );

        return {
            doctorId: room.doctorId,
            doctor: room.doctor,
            roomNumber: room.roomNumber,
            roomStatus: currentApp,
            currentPatient: currentApp
                ? `${currentApp.patient.firstName} ${currentApp.patient.lastName}`
                : "None",
            nextAppointment: nextApp
                ? `${nextApp.time} | ${nextApp.patient.firstName} ${nextApp.patient.lastName}`
                : "No upcoming"
        };
    });
}

function renderRoomTable() {
    const roomTableBody = document.querySelector("#roomMonitorTbl tbody");
    roomTableBody.innerHTML = "";
    const rooms = getRoomStatuses();

    rooms.forEach(room => {
        const row = document.createElement("tr");
        row.dataset.doc = room.doctor;
        row.dataset.roomStatus = room.roomStatus;

        row.innerHTML = `
            <td>${room.doctor}</td>
            <td>${room.roomNumber}</td>
            <td><span class="status ${room.roomStatus ? 'occupied' : 'vacant'}">${room.roomStatus ? 'OCCUPIED' : 'VACANT'}</span></td>
            <td>${room.currentPatient}</td>
            <td>${room.nextAppointment}</td>
        `;
        roomTableBody.appendChild(row);
    });
}

function getPatientStatus(app) {
    if (app.queueStatus === "Called" && app.currentAppointmentId === app.appointmentId) {
        const waitingMinutes = app.calledAt
            ? Math.max(0, Math.floor((Date.now() - new Date(app.calledAt).getTime()) / 60000))
            : 0;
        const text = waitingMinutes >= 10
            ? `Doctor delay: ${waitingMinutes} min`
            : "Waiting for Doctor";
        return { class: "status status-ready", text };
    }
    if (app.status === "In Consultation"
        && app.raw.consultation_deadline
        && Date.now() >= new Date(app.raw.consultation_deadline).getTime()) {
        return { class: "status status-inroom", text: "Consultation Overdue" };
    }
    switch (app.status) {
        case "Scheduled": return { class: "status status-notarrive", text: "Not Arrived" };
        case "Checked In": return { class: "status status-pendingvital", text: "Pending Vitals" };
        case "Vitals Recorded": return { class: "status status-ready", text: "Ready for Doctor" };
        case "In Consultation": return { class: "status status-inroom", text: "In Consultation" };
        case "Completed": return { class: "status status-completed", text: "Completed" };
        case "No Show": return { class: "status status-noshow", text: "No Show" };
        case "Cancelled": return { class: "status status-cancelled", text: "Cancelled" };
        default: return { class: "status", text: app.status };
    }
}

function getPriorityRank(priority) {
    if (priority === "Emergency") return 1;
    if (priority === "Urgent") return 2;
    return 3;
}

function sortByPriorityThenTime(list) {
    return list.sort((a, b) => {
        const priorityDiff = getPriorityRank(a.priority) - getPriorityRank(b.priority);
        if (priorityDiff !== 0) return priorityDiff;

        return new Date(a.raw.scheduled_at) - new Date(b.raw.scheduled_at);
    });
}

function getPriorityBadge(priority) {
    switch (priority) {
        case "Emergency":
            return { class: "badge badge-emergency", text: "Emergency" };
        case "Urgent":
            return { class: "badge badge-urgent", text: "Urgent" };
        default:
            return { class: "badge badge-normal", text: "Normal" };
    }
}

async function fetchAppointmentsByDate(dateStr) {
    try {
        const res = await fetch(`/api/dashboard/appointments?date=${dateStr}`);
        if (!res.ok) throw new Error("Failed to load appointments");
        const rawList = await res.json();
        // Convert the nested API response into the simpler shape used by the table.
        globalAppointments = rawList.map(row => {
            const queue = Array.isArray(row.patient_queue) ? row.patient_queue[0] : row.patient_queue;
            return {
                appointmentId: row.id,
                doctorId: row.doctor_id,
                date: extractIsoDate(row.scheduled_at),
                time: formatIsoTime(row.scheduled_at),
                status: row.status,
                queueStatus: queue?.status || "Not Checked In",
                calledAt: queue?.called_at || null,
                priority: queue?.priority || "Normal",
                priorityReason: queue?.priority_reason || "",
                room: row.doctors?.room || "Unassigned",
                roomStatus: row.doctors?.room_status?.[0]?.status || "Available",
                currentAppointmentId: row.doctors?.room_status?.[0]?.current_appointment_id || null,
                doctor: row.doctors?.staff?.full_name || "Unknown Doctor",
                patient: {
                    firstName: row.patients?.first_name || "",
                    lastName: row.patients?.last_name || "",
                    id: row.patients?.id || ""
                },
                raw: row
            };
        });
        globalAppointments = sortByPriorityThenTime(globalAppointments);
        return globalAppointments;
    } catch (err) {
        console.error(err);
        showToast("Failed to load appointment data", "danger");
        return [];
    }
}

function getDoctorRoom(doctorId) {
    return globalRooms.find(room => room.doctorId === doctorId);
}

function isDoctorRoomAvailable(doctorId) {
    const room = getDoctorRoom(doctorId);

    return room
        && room.roomStatus === "Available"
        && !room.currentAppointmentId;
}

function renderAppointmentRow(app) {
    const status = getPatientStatus(app);
    const fullName = `${app.patient.firstName} ${app.patient.lastName}`;
    const todayStr = getTodayDate();
    const isToday = app.date === todayStr;
    // Checked-in patients stay actionable even if the visit passed midnight.
    const isOverdue = app.date < todayStr && ["Checked In", "Vitals Recorded", "In Consultation"].includes(app.status);
    const canEnterVitals = app.status === "Checked In";
    const canSendRoom = app.status === "Vitals Recorded"
        && app.queueStatus === "Waiting"
        && isDoctorRoomAvailable(app.doctorId);
    const canArrive = isToday && app.status === "Scheduled";
    const priority = getPriorityBadge(app.priority);
    const displayedTime = isOverdue ? `${app.date} ${app.time}` : app.time;
    const statusText = isOverdue ? `Overdue: ${status.text}` : status.text;

    let html = `
        <tr data-doctor="${app.doctor}" data-status="${app.status}" data-app-id="${app.appointmentId}">
            <td>${fullName}</td>
            <td>${app.doctor}</td>
            <td>${displayedTime}</td>`;

    if (isToday || isOverdue) {
        html += `
        <td><span class="${status.class}">${statusText}</span></td>
        <td><span class="${priority.class}">${priority.text}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <button class="btn btn-arrive ${!canArrive ? 'disabled' : ''}" ${!canArrive ? 'disabled' : ''} onclick="handleMarkArrive('${app.appointmentId}')"><i class="fa-solid fa-user-check"></i></button>
                <span class="tooltip-text">Arrive</span>
            </div>
            <div class="has-tooltip">
                <button class="btn btn-vitals ${!canEnterVitals ? 'disabled' : ''}" ${!canEnterVitals ? 'disabled' : ''} onclick="openVital('${fullName}', '${app.appointmentId}')"><i class="fa-solid fa-file-waveform"></i></button>
                <span class="tooltip-text">Enter Vitals</span>
            </div>
            <div class="has-tooltip">
                <button class="btn btn-sendRoom ${!canSendRoom ? 'disabled' : ''}" ${!canSendRoom ? 'disabled' : ''} onclick="sendToRoom('${app.appointmentId}', '${fullName}', '${app.doctorId}')"><i class="fa-solid fa-arrow-right-from-bracket"></i></button>
                <span class="tooltip-text">Send to room</span>
            </div>
        </td>`;
    } else {
        html += `
            <td><span class="${status.class}">${status.text}</span></td>
            <td><span class="${priority.class}">${priority.text}</span></td>
            <td class="action">-</td>
        `;
    }

    html += `</tr>`;
    return html;
}

async function apiMarkArrived(appointmentId) {
    try {
        const res = await fetch("/api/dashboard/mark-arrived", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ appointment_id: appointmentId })
        });

        const data = await readApiResponse(res);

        if (!res.ok) {
            throw new Error(data || "Check-in failed");
        }

        showToast("Patient marked arrived successfully!", "success");
        await fetchRooms();
        await refreshAppointmentTable(currentSelectedDate);
        renderRoomTable();
    } catch (err) {
        showToast(err.message, "danger");
    }
}

async function apiSaveVitals(appointmentId, vitalsPayload) {
    try {
        const res = await fetch("/api/dashboard/save-vitals", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                appointment_id: appointmentId,
                bp: vitalsPayload.bp,
                temp: Number(vitalsPayload.temp),
                pulse: Number(vitalsPayload.pulse),
                height: Number(vitalsPayload.height),
                weight: Number(vitalsPayload.weight),
                priority: vitalsPayload.priority,
                priority_reason: vitalsPayload.priorityReason || null
            })
        });
        const data = await readApiResponse(res);

        if (!res.ok) {
            throw new Error(data || "Save failed");
        }
        showToast("Vitals saved, patient ready for doctor!", "success");
        await fetchRooms();
        await refreshAppointmentTable(currentSelectedDate);
        renderRoomTable();
        return true;
    } catch (err) {
        showToast(err.message, "danger");
        return false;
    }
}

async function refreshAppointmentTable(selectedDateStr, silent = false) {
    if (!silent) showToast("Loading appointments...", "loading");

    try {
        currentSelectedDate = selectedDateStr;
        await fetchAppointmentsByDate(selectedDateStr);

        pagination.data = globalAppointments;
        pagination.currentPage = 1;
        pagination.renderTable();

        if (globalAppointments.length === 0) {
            document.getElementById("appBody").innerHTML =
                `<tr><td colspan="6" class="empty-row">No appointments found for this date.</td></tr>`;
        }
        if (!silent) showToast("Appointments loaded!", "success");
    } catch (err) {
        if (!silent) showToast("Appointments could not be loaded!", "error");
    }
}

async function handleMarkArrive(appointmentId) {
    await apiMarkArrived(appointmentId);
}

let activeVitalAppId = "";
function openVital(patientName, appId) {
    activeVitalAppId = appId;
    document.getElementById("currentPatientName").textContent = patientName;
    // Clear vital form inputs
    document.getElementById("bp").value = "";
    document.getElementById("temp").value = "";
    document.getElementById("pulse").value = "";
    document.getElementById("height").value = "";
    document.getElementById("weight").value = "";
    openModal('vitalModal');
}

async function saveVital(button) {
    const payload = {
        bp: document.getElementById("bp").value,
        temp: document.getElementById("temp").value,
        pulse: document.getElementById("pulse").value,
        height: document.getElementById("height").value,
        weight: document.getElementById("weight").value,
        priority: document.getElementById("queuePriority").value,
        priorityReason: document.getElementById("priorityReason").value.trim()
    };

    // Wait for the server before closing so users can correct rejected values.
    const saved = await apiSaveVitals(activeVitalAppId, payload);
    if (saved) {
        closeModal(button);
        clearInput(button);
    }
}

function applyAllFiltersAndRefresh() {
    const searchKeyword = document.getElementById("searchInput").value.toLowerCase().trim();
    const selectedDoctor = document.getElementById("doctorFilter").value;
    const selectedStatus = document.getElementById("statusFilter").value;

    let filtered = globalAppointments.filter(p => p.date === currentSelectedDate);
    filtered = sortByPriorityThenTime(filtered);

    if (searchKeyword) {
        filtered = filtered.filter(item => {
            const fullName = `${item.patient.firstName} ${item.patient.lastName}`.toLowerCase();
            const doctorName = item.doctor.toLowerCase();
            return fullName.includes(searchKeyword) || doctorName.includes(searchKeyword);
        });
    }

    if (selectedDoctor !== "all") {
        filtered = filtered.filter(item => item.doctor === selectedDoctor);
    }

    if (selectedStatus !== "all") {
        filtered = filtered.filter(item => item.status === selectedStatus);
    }

    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();

    if (filtered.length === 0) {
        document.getElementById("appBody").innerHTML =
            `<tr><td colspan="6" class="empty-row">No appointments found.</td></tr>`;
    }
}

function initFilters() {
    const search = document.getElementById("searchInput");
    const doctor = document.getElementById("doctorFilter");
    const status = document.getElementById("statusFilter");

    search.addEventListener("input", applyAllFiltersAndRefresh);
    doctor.addEventListener("change", applyAllFiltersAndRefresh);
    status.addEventListener("change", applyAllFiltersAndRefresh);
}

async function apiSendToRoom(appointmentId, doctorId) {
    const res = await fetch("/api/dashboard/send-to-room", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            appointment_id: appointmentId,
            doctor_id: doctorId
        })
    });

    const data = await readApiResponse(res);

    if (!res.ok) {
        throw new Error(data || "Send to room failed");
    }
}

async function sendToRoom(appointmentId, patientName, doctorId) {
    try {
        await apiSendToRoom(appointmentId, doctorId);
        showToast(`${patientName} sent to consultation room`, "success");
        await fetchRooms();
        await refreshAppointmentTable(currentSelectedDate);
        renderRoomTable();
    } catch (err) {
        showToast(err.message, "danger");
    }
}

function initPrioritySelector() {
    const group = document.getElementById("queuePriorityGroup");
    const hiddenInput = document.getElementById("queuePriority");
    const reasonField = document.getElementById("priorityReasonField");
    const reasonInput = document.getElementById("priorityReason");

    if (!group || !hiddenInput || !reasonField || !reasonInput) return;

    group.querySelectorAll(".priority-tab").forEach(tab => {
        tab.addEventListener("click", () => {
            const selectedPriority = tab.dataset.priority;

            group.querySelectorAll(".priority-tab").forEach(item => {
                item.classList.remove("selected");
            });

            tab.classList.add("selected");
            hiddenInput.value = selectedPriority;

            const needsReason =
                selectedPriority === "Urgent" ||
                selectedPriority === "Emergency";

            reasonField.style.display = needsReason ? "flex" : "none";
            reasonInput.required = needsReason;

            if (!needsReason) {
                reasonInput.value = "";
            }
        });
    });
}


document.addEventListener("DOMContentLoaded", async () => {
    const todayStr = getTodayDate();
    currentSelectedDate = todayStr;
    await fetchRooms();
    await loadDoctors();
    renderRoomTable();
    initFilters();
    updateStatsCards();
    initPrioritySelector();
    flatpickr("#fullCalendar", {
        inline: true,
        dateFormat: "Y-m-d",
        defaultDate: todayStr,
        onChange: async (_, dateStr) => {
            await fetchRooms();
            await refreshAppointmentTable(dateStr);
            renderRoomTable();
        }
    });
    await fetchRooms();
    await refreshAppointmentTable(todayStr);
    renderRoomTable();

    // Refresh the queue so reception can see room and doctor delays promptly.
    setInterval(async () => {
        await fetchRooms();
        await refreshAppointmentTable(currentSelectedDate, true);
        renderRoomTable();
    }, 30000);
});
