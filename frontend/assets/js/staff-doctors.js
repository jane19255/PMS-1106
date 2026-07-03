let managedDoctors = [];
let offboardingDoctorId = null;

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
                <a href="/doctors/${encodeURIComponent(doctor.id)}" aria-label="View doctor details"><i class="fa-solid fa-circle-info"></i></a>
                <button class="offboard-doctor" type="button" data-doctor-id="${escapeHtml(doctor.id)}" aria-label="Offboard doctor"><i class="fa-solid fa-user-slash"></i></button>
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
    if (!confirm("Delete this doctor, their staff record, and login account? This only works if they have no appointment history at all — open the doctor's page to Offboard (reassign & deactivate) a doctor who has seen patients.")) return;
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

function openOffboardModal(doctorId) {
    const doctor = managedDoctors.find(candidate => candidate.id === doctorId);
    if (!doctor) return;

    offboardingDoctorId = doctorId;

    const description = document.getElementById("offboardDoctorDescription");
    description.textContent = `Move ${doctor.name}'s upcoming appointments to another doctor, then deactivate ${doctor.name}'s profile and login. In-progress visits must be completed or cancelled first; past visits stay recorded under ${doctor.name}.`;

    const select = document.getElementById("offboard-target-doctor");
    const otherDoctors = managedDoctors.filter(candidate => candidate.id !== doctorId);
    select.innerHTML = '<option value="" disabled selected>Select a doctor&hellip;</option>' +
        otherDoctors
            .map(candidate => {
                const statusLabel = candidate.status !== "Available" ? ` (${candidate.status})` : "";
                return `<option value="${candidate.id}">${escapeHtml(candidate.name)} — ${escapeHtml(candidate.specialization)}${statusLabel}</option>`;
            })
            .join("");

    const modal = document.getElementById("offboardDoctorModal");
    modal.querySelectorAll(".error-box").forEach(box => { box.style.display = "none"; });

    openModal("offboardDoctorModal");
}

async function submitOffboardDoctor() {
    const select = document.getElementById("offboard-target-doctor");
    const modal = document.getElementById("offboardDoctorModal");
    const errorBox = modal.querySelector(".error-box");

    if (!select.value) {
        errorBox.innerHTML = '<i class="fa-solid fa-circle-exclamation"></i> Select a doctor to reassign appointments to.';
        errorBox.style.display = "flex";
        return;
    }

    if (!confirm("Reassign this doctor's upcoming appointments and deactivate their account? This does not delete any records.")) return;

    try {
        const response = await fetch(`/api/doctors/${encodeURIComponent(offboardingDoctorId)}/reassign`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ target_doctor_id: select.value })
        });

        if (!response.ok) {
            const message = await response.text();
            errorBox.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${escapeHtml(message)}`;
            errorBox.style.display = "flex";
            return;
        }

        modal.style.display = "none";
        document.documentElement.classList.remove("no-scroll");
        showToast("Doctor offboarded and appointments reassigned.", "success");
        await loadDoctors();
    } catch (error) {
        errorBox.innerHTML = `<i class="fa-solid fa-circle-exclamation"></i> ${escapeHtml(error.message || "Could not reach the server.")}`;
        errorBox.style.display = "flex";
    }
}

document.addEventListener("DOMContentLoaded", () => {
    loadDoctors();

    if (typeof toggleAddDoctorFields === "function") toggleAddDoctorFields();

    document.getElementById("doctorTableBody").addEventListener("click", event => {
        const deleteButton = event.target.closest(".delete-doctor");
        if (deleteButton) deleteDoctor(deleteButton.dataset.doctorId);

        const offboardButton = event.target.closest(".offboard-doctor");
        if (offboardButton) openOffboardModal(offboardButton.dataset.doctorId);
    });

});
