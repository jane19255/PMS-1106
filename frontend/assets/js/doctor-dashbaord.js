let appointments = [];
let currentSelectedDate = getTodayDate();

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

function getActiveAppointments() {
    return appointments;
}

function getTodayDate() {
    return new Intl.DateTimeFormat("en-CA", {
        timeZone: "Asia/Singapore",
        year: "numeric",
        month: "2-digit",
        day: "2-digit"
    }).format(new Date());
}

function getQueueStatus(status) {
    switch (status) {
        case "Waiting": return { class: "status status-pendingvital", text: "Waiting" };
        case "Called": return { class: "status status-ready", text: "Waiting in room" };
        case "In Consultation": return { class: "status status-inroom", text: "In Consultation" };
        case "Completed": return { class: "status status-completed", text: "Completed" };
        default: return { class: "status", text: status || "Not Checked In" };
    }
}

function getPriorityBadge(priority) {
    switch (priority) {
        case "Emergency": return { class: "badge badge-emergency", text: "Emergency" };
        case "Urgent": return { class: "badge badge-urgent", text: "Urgent" };
        default: return { class: "badge badge-normal", text: "Normal" };
    }
}

function isConsultationTimeUp(app) {
    return app.queueStatus === "In Consultation"
        && !!app.consultationDeadline
        && Date.now() >= new Date(app.consultationDeadline).getTime();
}

function consultationOverrunText(app) {
    const deadline = new Date(app.consultationDeadline).getTime();
    const minutes = Math.max(0, Math.floor((Date.now() - deadline) / 60000));
    return minutes === 0 ? "Time up" : `Overdue by ${minutes} min`;
}

function renderAppointmentRow(app) {
    const status = getQueueStatus(app.queueStatus);
    const priority = getPriorityBadge(app.priority);
    // Show the original date so the doctor can identify a carried-forward visit.
    const isOverdue = app.appointmentDate < getTodayDate();
    const displayedTime = isOverdue
        ? `${app.appointmentDate} ${app.appointmentTime}`
        : app.appointmentTime;
    const timeUp = isConsultationTimeUp(app);
    const statusText = timeUp
        ? consultationOverrunText(app)
        : (isOverdue ? `Overdue: ${status.text}` : status.text);
    const statusClass = timeUp ? "status status-timeup" : status.class;

    let actionHtml = `<span class="muted">-</span>`;

    if (app.queueStatus === "Called") {
        actionHtml = `
            <button onclick="event.stopPropagation(); startConsultation('${app.appointmentId}')">
                Start Consultation
            </button>
        `;
    } else if (app.queueStatus === "In Consultation" && timeUp) {
        actionHtml = `
            <div class="timeup-actions" onclick="event.stopPropagation();">
                <button class="secondary" onclick="extendConsultation('${app.appointmentId}')">
                    Extend 15 min
                </button>
                <button class="danger" onclick="completeConsultation('${app.appointmentId}')">
                    End &amp; free room
                </button>
            </div>
        `;
    } else if (app.queueStatus === "In Consultation") {
        actionHtml = `
            <button onclick="event.stopPropagation(); completeConsultation('${app.appointmentId}')">
                Complete
            </button>
        `;
    }

    return `
        <tr onclick="window.location.href='/medical-records?patient_id=${app.patientId}'">
            <td>${app.patientName}</td>
            <td>${displayedTime}</td>
            <td><span class="${statusClass}">${statusText}</span></td>
            <td><span class="${priority.class}">${priority.text}</span></td>
            <td class="action">${actionHtml}</td>
        </tr>
    `;
}

async function refreshAppointmentTable(selectedDate) {
    currentSelectedDate = selectedDate;
    await loadDashboardAppointments(selectedDate);

    pagination.data = appointments;
    pagination.currentPage = 1;
    pagination.renderTable();

    if (appointments.length === 0) {
        document.getElementById("appBody").innerHTML =
            `<tr><td colspan="5" class="empty-row">No appointments found for this date.</td></tr>`;
    }
}

function applyAllFiltersAndRefresh() {
    const searchKeyword = document.getElementById("searchInput")?.value.toLowerCase().trim() || "";
    const selectedQueueStatus = document.getElementById("queueStatusFilter")?.value || "all";

    let filtered = appointments;

    if (searchKeyword) {
        filtered = filtered.filter(item =>
            item.patientName.toLowerCase().includes(searchKeyword)
        );
    }

    if (selectedQueueStatus !== "all") {
        filtered = filtered.filter(item => item.queueStatus === selectedQueueStatus);
    }

    pagination.data = filtered;
    pagination.currentPage = 1;
    pagination.renderTable();

    if (filtered.length === 0) {
        document.getElementById("appBody").innerHTML =
            `<tr><td colspan="5" class="empty-row">No appointments found for this date.</td></tr>`;
    }
}

function initFilters() {
    const search = document.getElementById("searchInput");
    const queueStatus = document.getElementById("queueStatusFilter");

    if (search) {
        search.addEventListener("input", applyAllFiltersAndRefresh);
    }

    if (queueStatus) {
        queueStatus.addEventListener("change", applyAllFiltersAndRefresh);
    }
}

async function loadDashboardAppointments(dateStr = currentSelectedDate) {
    try {
        const response = await fetch(`/api/doctor-dashboard/appointments?date=${dateStr}`, {
            headers: { Accept: "application/json" },
        });

        if (!response.ok) throw new Error(`Dashboard API returned ${response.status}`);

        appointments = await response.json();
    } catch (error) {
        console.warn("Could not load doctor dashboard appointments:", error);
        appointments = [];
        showToast("Doctor appointments could not be loaded", "danger");
    }
}

async function startConsultation(appointmentId) {
    try {
        const res = await fetch("/api/doctor-dashboard/start-consultation", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ appointment_id: appointmentId })
        });

        if (!res.ok) throw new Error(await res.text());

        showToast("Consultation started", "success");
        await refreshAppointmentTable(currentSelectedDate);
    } catch (error) {
        showToast(error.message || "Consultation could not be started", "error");
    }
}

async function extendConsultation(appointmentId) {
    try {
        const res = await fetch("/api/doctor-dashboard/extend-consultation", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ appointment_id: appointmentId })
        });

        if (!res.ok) throw new Error(await res.text());

        showToast("Consultation extended by 15 minutes", "success");
        await refreshAppointmentTable(currentSelectedDate);
    } catch (error) {
        showToast(error.message || "Consultation could not be extended", "error");
    }
}

async function completeConsultation(appointmentId) {
    try {
        const res = await fetch("/api/doctor-dashboard/complete-consultation", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ appointment_id: appointmentId })
        });

        if (!res.ok) throw new Error(await res.text());

        showToast("Consultation completed", "success");
        await refreshAppointmentTable(currentSelectedDate);
    } catch (error) {
        showToast(error.message || "Consultation could not be completed", "error");
    }
}

document.addEventListener("DOMContentLoaded", async () => {
    initFilters();

    flatpickr("#fullCalendar", {
        inline: true,
        dateFormat: "Y-m-d",
        defaultDate: getTodayDate(),
        onChange: async (_, dateStr) => {
            await refreshAppointmentTable(dateStr);
        }
    });

    await refreshAppointmentTable(getTodayDate());

    // Fetch again so room and consultation changes from other users stay visible.
    setInterval(() => refreshAppointmentTable(currentSelectedDate), 30000);
});
