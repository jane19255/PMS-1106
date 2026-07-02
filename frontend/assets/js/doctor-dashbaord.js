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
    const today = new Date();
    const year = today.getFullYear();
    const month = String(today.getMonth() + 1).padStart(2, '0');
    const day = String(today.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
}

function getQueueStatus(status) {
    switch (status) {
        case "Waiting": return { class: "status status-pendingvital", text: "Waiting" };
        case "Called": return { class: "status status-ready", text: "Ready to consult" };
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

function renderAppointmentRow(app) {
    const status = getQueueStatus(app.queueStatus);
    const priority = getPriorityBadge(app.priority);

    let actionHtml = `<span class="muted">-</span>`;

    if (app.queueStatus === "Called") {
        actionHtml = `
            <button onclick="event.stopPropagation(); startConsultation('${app.appointmentId}')">
                Start Consultation
            </button>
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
            <td>${app.appointmentTime}</td>
            <td><span class="${status.class}">${status.text}</span></td>
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
    const res = await fetch("/api/doctor-dashboard/start-consultation", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ appointment_id: appointmentId })
    });

    if (!res.ok) throw new Error(await res.text());

    showToast("Consultation started", "success");
    await refreshAppointmentTable(currentSelectedDate);
}

async function completeConsultation(appointmentId) {
    const res = await fetch("/api/doctor-dashboard/complete-consultation", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ appointment_id: appointmentId })
    });

    if (!res.ok) throw new Error(await res.text());

    showToast("Consultation completed", "success");
    await refreshAppointmentTable(currentSelectedDate);
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
});
