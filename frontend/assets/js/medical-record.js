function addNewPresRow() {
    let wrap = document.getElementById("presWrap");
    let newRow = document.createElement("div");
    newRow.className = "pres-row";
    newRow.innerHTML = `
        <div class="field">
            <label class="required">Medicine</label>
            <div class="custom-select">
                <i class="fa-solid fa-capsules"></i>
                <select class="med-select">
                    <option>Paracetamol</option>
                    <option>Amoxicillin</option>
                    <option>Ibuprofen</option>
                    <option>Cetirizine</option>
                </select>
            </div>
        </div>
        <div class="field">
            <label class="required">Dosage</label>
            <div class="custom-select">
                <i class="fa-solid fa-prescription-bottle-medical"></i>
                <select class="dosage-select">
                    <option>250mg</option>
                    <option>500mg</option>
                    <option>1000mg</option>
                </select>
            </div>
        </div>
        <div class="field">
            <label class="required">Frequency</label>
            <div class="custom-select">
                <i class="fa-solid fa-prescription-bottle"></i>
                <select class="freq-select">
                    <option>1x/Day</option>
                    <option>2x/Day</option>
                    <option>3x/Day</option>
                </select>
            </div>
        </div>
        <div class="field">
            <label class="required">Duration</label>
            <input class="dur-input" type="text" placeholder="e.g. 5 days" value="5 days">
        </div>
        <div class="field">
            <label class="required">Qty</label>
            <input class="qty-input" type="number" placeholder="10" value="10" min="1">
        </div>
    `;
    wrap.appendChild(newRow);
    showToast("New empty prescription row added", "success");
}

function showAddPrescriptTab(cb) {
    const tab = document.getElementById("addPrescriptTab");
    const inputs = tab.querySelectorAll('input, textarea, select');
    const labels = tab.querySelectorAll("label");
    tab.classList.toggle('show');

    inputs.forEach(input => {
        if (!cb.checked) {
            if (input.type === 'checkbox' || input.type === 'radio') {
                input.checked = false;
            }
            input.required = false;
        } else {
            input.required = true;
        }
    });

    const rows = document.querySelectorAll('#presWrap .pres-row');
    for (let i = 1; i < rows.length; i++) {
        rows[i].remove();
    }
};

function toggleHistory(el) {
    let body = el.nextElementSibling;
    let icon = el.querySelector(".fa-chevron-down");
    body.classList.toggle("open");
    if (body.classList.contains("open")) icon.style.transform = "rotate(180deg)";
    else icon.style.transform = "rotate(0deg)";
}

function searchPatient() {
    const id = document.getElementById('patientIdInput').value.trim();
    if (!id) return;
    const url = new URL(window.location.href);
    url.searchParams.set('patient_id', id);
    window.history.replaceState({}, '', url.toString());
    loadPatient(id);
    loadHistory(id);
}

async function loadPatient(patientId) {
    try {
        const res = await fetch(`/api/patients/${patientId}`);
        if (!res.ok) { showToast('Patient not found', 'error'); return; }
        const p = await res.json();
        document.getElementById('pat-name').textContent = `${p.firstName} ${p.lastName}`;
        document.getElementById('pat-ic').textContent = p.nric || '—';
        document.getElementById('pat-dob').textContent = p.dob || '—';
        document.getElementById('pat-phone').textContent = p.phone || '—';
        document.getElementById('pat-allergies').textContent = p.allergies || 'None';
        document.getElementById('pat-medications').textContent = p.medications || 'None';
        document.getElementById('pat-conditions').textContent = p.conditions || 'None';
    } catch (e) {
        console.error('Failed to load patient:', e);
    }
}

async function loadHistory(patientId) {
    const container = document.getElementById('prevRecord');
    const title = container.querySelector('.title');

    try {
        const [histRes, presRes] = await Promise.all([
            fetch(`/api/patients/${encodeURIComponent(patientId)}/history`),
            fetch(`/api/prescriptions?patient_id=${encodeURIComponent(patientId)}`)
        ]);

        container.innerHTML = '';
        if (title) container.appendChild(title);

        if (!histRes.ok) {
            container.insertAdjacentHTML('beforeend', '<p style="color:#888;padding:16px 0;font-size:14px">Could not load visit history.</p>');
            return;
        }

        const entries = await histRes.json();
        const prescriptions = presRes.ok ? await presRes.json() : [];

        // Group prescriptions by medical_record_id
        const presMap = {};
        for (const p of prescriptions) {
            if (p.medical_record_id) {
                if (!presMap[p.medical_record_id]) presMap[p.medical_record_id] = [];
                presMap[p.medical_record_id].push(p);
            }
        }

        if (!entries.length) {
            container.insertAdjacentHTML('beforeend', '<p style="color:#888;padding:16px 0;font-size:14px">No previous visits recorded.</p>');
            return;
        }

        entries.forEach(entry => container.appendChild(buildHistoryCard(entry, presMap[entry.id] || [])));
    } catch (e) {
        console.error('Failed to load history:', e);
    }
}

function buildHistoryCard(entry, prescriptions) {
    const date = entry.recorded_at ? new Date(entry.recorded_at).toLocaleDateString('en-GB') : '—';
    const doctor = entry.doctor_name || entry.doctor_id || '—';

    const findingsDiag = [entry.clinical_findings, entry.diagnosis].filter(Boolean).join(' — ') || '—';

    let presLines = '';
    if (prescriptions && prescriptions.length > 0) {
        presLines = prescriptions
            .map(p => `<p>${p.medication_name} ${p.dosage} ${p.frequency}${p.duration ? ', ' + p.duration : ''}</p>`)
            .join('');
    } else {
        presLines = '<p>—</p>';
    }

    const card = document.createElement('div');
    card.className = 'history-card';
    card.innerHTML = `
        <div class="history-head" onclick="toggleHistory(this)">
            <span>Visit Date: ${date}<span>${doctor}</span></span>
            <div>
                <button onclick="event.stopPropagation();printSingleHist(this)">Print This Visit</button>
                <i class="fas fa-chevron-down"></i>
            </div>
        </div>
        <div class="history-body">
            <div class="hist-data">
                <div><p>BP</p><p>${entry.blood_pressure || '—'}</p></div>
                <div><p>Temp</p><p>${entry.temperature ? entry.temperature + '°C' : '—'}</p></div>
                <div><p>Pulse</p><p>${entry.pulse_rate ? entry.pulse_rate + 'bpm' : '—'}</p></div>
                <div><p>Height</p><p>${entry.height_cm ? entry.height_cm + 'cm' : '—'}</p></div>
                <div><p>Weight</p><p>${entry.weight_kg ? entry.weight_kg + 'kg' : '—'}</p></div>
            </div>
            <div class="hist-block">
                <p>Reason: </p>
                <p>${entry.reason_of_visit || '—'}</p>
            </div>
            <div class="hist-block">
                <p>Findings &amp; Diagnosis: </p>
                <p>${findingsDiag}</p>
            </div>
            <div class="hist-block">
                <p>Notes: </p>
                <p>${entry.doctor_notes || '—'}</p>
            </div>
            <div class="hist-block">
                <p>Prescription:</p>
                ${presLines}
            </div>
        </div>`;
    return card;
}

async function saveRecord(patientId) {
    const doctorId = document.getElementById('visitDoctorId').value.trim() || null;
    const body = {
        patient_id: patientId,
        blood_pressure: document.getElementById('inp-bp').value.trim() || null,
        temperature: document.getElementById('inp-temp').value.trim() || null,
        pulse_rate: document.getElementById('inp-pulse').value.trim() || null,
        height_cm: document.getElementById('inp-height').value.trim() || null,
        weight_kg: document.getElementById('inp-weight').value.trim() || null,
        reason_of_visit: document.getElementById('visitReason').value.trim() || null,
        clinical_findings: document.getElementById('findings').value.trim() || null,
        diagnosis: document.getElementById('diagnosis').value.trim() || null,
        doctor_notes: document.getElementById('treatNote').value.trim() || null,
        doctor_id: doctorId,
    };

    try {
        const res = await fetch('/api/medical-records', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (res.ok) {
            const savedRecord = await res.json();

            const addPrescriptCb = document.getElementById('addPrescript');
            if (addPrescriptCb && addPrescriptCb.checked && savedRecord.id) {
                const presOk = await savePrescriptions(patientId, doctorId, savedRecord.id);
                if (!presOk) {
                    // Rollback: delete the medical record so nothing is partially saved
                    await fetch(`/api/medical-records/${encodeURIComponent(savedRecord.id)}`, { method: 'DELETE' });
                    showToast('Prescription failed — visit record was not saved.', 'error');
                    return;
                }
            }

            showToast('Record saved successfully!', 'success');
            ['visitReason', 'findings', 'diagnosis', 'treatNote'].forEach(id => {
                const el = document.getElementById(id);
                if (el) el.value = '';
            });
            ['inp-bp', 'inp-temp', 'inp-pulse', 'inp-height', 'inp-weight'].forEach(id => {
                const el = document.getElementById(id);
                if (el) el.value = '';
            });
            await loadHistory(patientId);
        } else {
            const text = await res.text();
            showToast('Save failed: ' + text, 'error');
        }
    } catch (e) {
        showToast('Network error: ' + e.message, 'error');
    }
}

async function savePrescriptions(patientId, doctorId, medicalRecordId) {
    const rows = document.querySelectorAll('#presWrap .pres-row');
    const instructions = document.getElementById('usageIns')?.value.trim() || null;
    let saved = 0;
    const errors = [];
    for (const row of rows) {
        const medicationName = row.querySelector('.med-select')?.value;
        const dosage = row.querySelector('.dosage-select')?.value;
        const frequency = row.querySelector('.freq-select')?.value;
        const duration = row.querySelector('.dur-input')?.value.trim() || '5 days';
        const quantity = row.querySelector('.qty-input')?.value.trim() || '10';
        if (!medicationName || !dosage || !frequency) continue;
        try {
            const res = await fetch('/api/prescriptions', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    patient_id: patientId,
                    doctor_id: doctorId || '',
                    medical_record_id: medicalRecordId,
                    medication_name: medicationName,
                    dosage,
                    frequency,
                    duration,
                    instructions: instructions || null,
                    quantity,
                    unit_cost: '0',
                }),
            });
            if (res.ok) saved++;
            else errors.push(`${medicationName}: ${await res.text()}`);
        } catch (e) {
            errors.push(`${medicationName}: network error`);
        }
    }
    if (errors.length > 0) {
        showToast('Prescription error: ' + errors[0], 'error');
        return false;
    }
    return true;
}

document.addEventListener('DOMContentLoaded', () => {
    const params = new URLSearchParams(window.location.search);
    const patientId = params.get('patient_id');
    if (patientId) {
        document.getElementById('patientIdInput').value = patientId;
        loadPatient(patientId);
        loadHistory(patientId);
    }

    const saveBtn = document.getElementById('saveVisitBtn');
    if (saveBtn) {
        saveBtn.addEventListener('click', () => {
            const id = new URLSearchParams(window.location.search).get('patient_id');
            if (!id) { showToast('Please load a patient first', 'error'); return; }
            saveRecord(id);
        });
    }
});

function printSingleHist(el) {
    // Fallback safety check
    let recordCard;
    if (el && el.closest) {
        recordCard = el.closest(".history-card");
    } else {
        recordCard = document.querySelector(".history-card");
    }

    if (!recordCard) {
        console.error("No history card found to print!");
        return;
    }

    let patientInfo = document.querySelector("#patientInfo")

    // Create hidden iframe for printing
    const iframe = document.createElement("iframe");
    iframe.style.position = "absolute";
    iframe.style.left = "-9999px";
    iframe.style.top = "-9999px";
    iframe.style.width = "0";
    iframe.style.height = "0";
    document.body.appendChild(iframe);

    const doc = iframe.contentDocument || iframe.contentWindow.document;
    doc.open();

    // Write the modern A4 medical record template
    doc.write(`
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Patient Medical Record</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
                font-family: 'Segoe UI', 'Arial', sans-serif;
            }

            @page {
                size: A4;
                margin: 20mm;
                @top-left { content: ""; }
                @top-right { content: ""; }
                @bottom-left { content: ""; }
                @bottom-right { content: ""; }
            }

            body {
                background: #ffffff;
                padding: 0;
                line-height: 1.5;
            }

            /* --- Hospital Header --- */
            .hospital-header {
                text-align: right;
                font-size: 14px;
                margin-bottom: 16px;
            }

            .hospital-header strong {
                display: block;
                font-size: 18px;
            }

            /* --- Main Title --- */
            .record-title {
                color: #084fde; 
                font-size: 18pt;
                font-weight: 700;
                margin-bottom: 20px;
                border-bottom: 1px solid #e2e8f0;
                padding-bottom: 8px;
            }

            /* --- Content Area --- */
            .info-grid {
                display: grid;
                grid-template-columns: repeat(4, minmax(0, 1fr));
                gap: 12px;
                margin-bottom: 36px;
            }
            .history-head, .hist-block {
                margin-bottom: 16px;
            }
            .history-head span:last-child {
                margin-left: 60%;
            }
            p:first-child  {font-weight: 700;}

            .history-head button, .hist-data, img {display:none;}

            /* --- Signature Area --- */
            .signature-container {
                margin-top: 34px;
                display: flex;
                justify-content: space-between;
                padding-top: 20px;
                border-top: 1px solid #e2e8f0;
            }

            .signature-line {
                padding-top: 6px;
                font-size: 14px;
            }

            /* --- Footer --- */
            .footer {
                position: absolute;
                bottom: 0;
                font-size: 12px;
                color: #718096;
                text-align: center;
            }
        </style>
    </head>
    <body>
        <!-- Hospital Header -->
        <div class="hospital-header">
            <strong>CSCare Clinic</strong>
            1 Punggol Coast Road<br>
            Singapore 828608<br>
            Phone: +65 9123 5678 | Email: info@cscare.com<br>
            Website: www.cscare.com
        </div>

        <div class="record-title">Patient Info</div>
        ${patientInfo.innerHTML}

        <div class="record-title">Medical Record</div>
        ${recordCard.innerHTML}

        <!-- Signature Area -->
        <div class="signature-container">
            <div class="signature-line">
                Physician Signature: ____________________
            </div>
            <div class="signature-line">
                Date: ____________________
            </div>
        </div>

        <!-- Footer -->
        <div class="footer">
            For appointments, billing inquiries, or medical assistance, please contact us at (123) 456-7890 or email info@cscare.com.<br>
            Thank you for choosing CSCare Hospital for your healthcare needs.
        </div>
    </body>
    </html>
    `);
    doc.close();

    // Trigger print dialog
    setTimeout(() => {
        iframe.contentWindow.print();
        // Clean up the iframe after print dialog closes
        setTimeout(() => {
            document.body.removeChild(iframe);
        }, 600);
    }, 200);
}