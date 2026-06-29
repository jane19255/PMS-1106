let managedDoctors = [];

function doctorStatusText(status) {
    return String(status || "Unavailable").replace(/([a-z])([A-Z])/g, "$1 $2");
}

function doctorStatusClass(status) {
    return doctorStatusText(status).toLowerCase().replaceAll(" ", "-");
}

function updateDoctorMetrics() {
    const statuses = managedDoctors.map(doctor => doctorStatusText(doctor.status).toLowerCase());
    document.getElementById("totalDoctorCount").textContent = managedDoctors.length;
    document.getElementById("availableDoctorCount").textContent = statuses.filter(status => status === "available").length;
    document.getElementById("unavailableDoctorCount").textContent = statuses.filter(status => status !== "available").length;
    document.getElementById("doctorSpecialtyCount").textContent = new Set(managedDoctors.map(doctor => doctor.specialization).filter(Boolean)).size;
}

function refreshDoctorList() {
    const body = document.getElementById("doctorTableBody");
    const search = document.getElementById("doctorSearchInput").value.trim().toLowerCase();
    const status = document.getElementById("doctorStatusFilter").value;
    const sort = document.getElementById("doctorSort").value;
    const doctors = managedDoctors
        .filter(doctor => {
            const doctorStatus = doctorStatusText(doctor.status).toLowerCase();
            const searchable = [doctor.id, doctor.staff_id, doctor.name, doctor.specialization, doctor.license_number]
                .join(" ").toLowerCase();
            return searchable.includes(search) && (status === "all" || doctorStatus === status);
        })
        .sort((left, right) => String(left[sort] || "").localeCompare(String(right[sort] || "")));

    if (!doctors.length) {
        body.innerHTML = '<tr><td class="empty-row" colspan="7">No matching doctors found.</td></tr>';
        return;
    }

    body.innerHTML = doctors.map(doctor => {
        const statusText = doctorStatusText(doctor.status);
        return `<tr>
            <td><strong>${escapeHtml(doctor.name)}</strong><br><span class="description">${escapeHtml(doctor.id)}</span></td>
            <td>${escapeHtml(doctor.staff_id)}</td>
            <td>${escapeHtml(doctor.license_number)}</td>
            <td>${escapeHtml(doctor.specialization)}</td>
            <td>${escapeHtml(doctor.contact_number)}<br><span class="description">${escapeHtml(doctor.email)}</span></td>
            <td><span class="doctor-status ${doctorStatusClass(doctor.status)}">${escapeHtml(statusText)}</span></td>
            <td><div class="doctor-actions">
                <a href="/doctors/${encodeURIComponent(doctor.id)}" aria-label="View doctor details"><i class="fa-solid fa-eye"></i></a>
                <button class="delete-doctor" type="button" data-doctor-id="${escapeHtml(doctor.id)}" aria-label="Delete doctor"><i class="fa-solid fa-trash"></i></button>
            </div></td>
        </tr>`;
    }).join("");
}

async function loadDoctors() {
    try {
        const response = await fetch("/api/doctors", { headers: { Accept: "application/json" } });
        if (!response.ok) throw new Error(await response.text());
        managedDoctors = await response.json();
        updateDoctorMetrics();
        refreshDoctorList();
    } catch (error) {
        document.getElementById("doctorTableBody").innerHTML = `<tr><td class="empty-row" colspan="7">${escapeHtml(error.message || "Could not load doctors.")}</td></tr>`;
    }
}

async function deleteDoctor(doctorId) {
    if (!confirm("Delete this doctor, their staff record, and login account? Existing appointments may prevent deletion.")) return;
    const response = await fetch(`/api/doctors/${encodeURIComponent(doctorId)}`, { method: "DELETE" });
    if (!response.ok) {
        showToast(await response.text(), "error");
        return;
    }
    managedDoctors = managedDoctors.filter(doctor => doctor.id !== doctorId);
    updateDoctorMetrics();
    refreshDoctorList();
    showToast("Doctor deleted successfully.", "success");
}

document.addEventListener("DOMContentLoaded", () => {
    loadDoctors();

    if (typeof toggleAddDoctorFields === "function") toggleAddDoctorFields();

    document.getElementById("doctorTableBody").addEventListener("click", event => {
        const button = event.target.closest(".delete-doctor");
        if (button) deleteDoctor(button.dataset.doctorId);
    });

});
