// Sample Billing Data (added extra fields for new print IDs)
const billingList = [
    {
        invoiceId: "INV-001",
        appointmentId: "APP-001",
        patientName: "Off Jumpol",
        patientId: "1",
        doctorName: "Dr. Jimmy Jitaraphol",
        billDate: "2026-06-03",
        billDateTime: "03/06/2026",
        status: "Pending",
        consultDesc: "Routine Checkup Consultation",
        consultDate: "03/06/2026",
        consultFee: "$1.00",
        medicineSubtotal: "$1.00",
        totalAmount: "$2.00",
        medicines: [
            { name: "Paracetamol 500mg Tablet", qty: 10, price: "$1.00", medDate: "03/06/2026" }
        ]
    },
    {
        invoiceId: "INV-002",
        appointmentId: "APP-002",
        patientName: "Gun Atthaphan",
        patientId: "2",
        doctorName: "Dr. Jimmy Jitaraphol",
        billDate: "2026-06-03",
        billDateTime: "03/06/2026",
        status: "Paid",
        consultDesc: "Follow-up Consultation",
        consultDate: "03/06/2026",
        consultFee: "$5.00",
        medicineSubtotal: "$10.00",
        totalAmount: "$15.00",
        medicines: [
            { name: "Amoxicillin 250mg Capsule", qty: 14, price: "$6.00", medDate: "03/06/2026" },
            { name: "Cetirizine 10mg Tablet", qty: 7, price: "$4.00", medDate: "03/06/2026" }
        ]
    },
    {
        invoiceId: "INV-9df00cef-4549-438e-9ee6-92cca5a0a923",
        appointmentId: "APP-003",
        patientName: "Junior Panuwat",
        patientId: "3",
        doctorName: "Dr. Jimmy Jitaraphol",
        billDate: "2026-06-03",
        billDateTime: "03/06/2026",
        status: "Overdue",
        consultDesc: "New Consultation",
        consultDate: "03/06/2026",
        consultFee: "$8.00",
        medicineSubtotal: "$12.00",
        totalAmount: "$20.00",
        medicines: [
            { name: "Ibuprofen 400mg Tablet", qty: 20, price: "$12.00", medDate: "03/06/2026" }
        ]
    }
];

let currentViewIndex = null;

const pagination = new Pagination({
    data: billingList,
    rowsPerPage: 3,
    tbodyId: "billingTableBody",
    pageInfoId: "pageInfo",
    pageSelectId: "pageSelect",
    prevBtnId: "prevBtn",
    nextBtnId: "nextBtn",
    renderRow: renderBillingRow
});

function renderBillingRow(item, index) {
    return `
    <tr>
        <td>${item.invoiceId}</td>
        <td>${item.patientName}</td>
        <td>${item.appointmentId}</td>
        <td>${item.billDate}</td>
        <td>${item.totalAmount}</td>
        <td><span class="status ${item.status}">${item.status}</span></td>
        <td class="action">
            <div class="has-tooltip">
                <i class="view fa-solid fa-circle-info" onclick="viewBill(${index})"></i>
                <span class="tooltip-text">View Details</span>
            </div>
            <div class="has-tooltip">
                <i class="print fa-solid fa-print" onclick="printSingleBill(${index})"></i>
                <span class="tooltip-text">Print Bill</span>
            </div>
            <div class="has-tooltip">
                <i class="record fa-solid fa-file-medical" onclick="jumpToRecord(${index})"></i>
                <span class="tooltip-text">Medical Record</span>
            </div>
        </td>
    </tr>
    `;
}

function parseDate(dateStr) {
    return new Date(dateStr);
}

function filterByDateRange(list, dateRange) {
    if (!dateRange || dateRange === "") return list;
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const oneWeekLater = new Date(today);
    oneWeekLater.setDate(today.getDate() + 7);
    const thisMonth = new Date(today.getFullYear(), today.getMonth() + 1, 0);

    return list.filter(item => {
        const billDate = parseDate(item.billDate);
        billDate.setHours(0, 0, 0, 0);
        switch (dateRange) {
            case "today":
                return billDate.getTime() === today.getTime();
            case "thisweek":
                return billDate >= today && billDate < oneWeekLater;
            case "thismonth":
                return billDate.getMonth() === today.getMonth() && billDate.getFullYear() === today.getFullYear();
            default:
                return true;
        }
    });
}

function refreshBillingList() {
    let list = [...billingList];
    const keyword = document.getElementById("searchInput")?.value.toLowerCase() || "";
    if (keyword) {
        list = list.filter(b =>
            b.invoiceId.toLowerCase().includes(keyword) ||
            b.patientName.toLowerCase().includes(keyword) ||
            b.appointmentId.toLowerCase().includes(keyword)
        );
    }

    const status = document.getElementById("filter-status")?.value;
    if (status) list = list.filter(b => b.status);

    const dateRange = document.getElementById("filter-date")?.value;
    list = filterByDateRange(list, dateRange);

    const sort = document.getElementById("sortBy").value;
    list.sort((a, b) => {
        if (sort === "dateNewest") return parseDate(b.billDate) - parseDate(a.billDate);
        if (sort === "dateOldest") return parseDate(a.billDate) - parseDate(b.billDate);
        if (sort === "status") return a.status.localeCompare(b.status);
        return 0;
    });

    pagination.data = list;
    pagination.currentPage = 1;
    pagination.renderTable();
}

function viewBill(index) {
    currentViewIndex = index;
    const item = billingList[index];
    openModal("detailsModal");

    document.getElementById("view-invoice-id").innerText = item.invoiceId;
    document.getElementById("view-appt-id").innerText = item.appointmentId;
    document.getElementById("view-patient").innerText = item.patientName;
    document.getElementById("view-bill-date").innerText = item.billDate;
    document.getElementById("view-status").innerText = item.status;

    document.getElementById("view-consult-desc").innerText = item.consultDesc;
    document.getElementById("view-consult-fee").innerText = item.consultFee;

    let medHtml = "";
    item.medicines.forEach(med => {
        medHtml += `<div class="row"><span>${med.name} (Qty: ${med.qty})</span> <span>${med.price}</span></div>`;
    });
    document.getElementById("medicineList").innerHTML = medHtml;
    document.getElementById("view-medicine-sub").innerText = item.medicineSubtotal;
    document.getElementById("view-total").innerText = item.totalAmount;

    fillPrintTemplate(item);
}

// ========== Updated fillPrintTemplate (use new IDs only) ==========
function fillPrintTemplate(item) {
    // Basic top info
    document.getElementById("print-payor").innerText = item.patientName;
    document.getElementById("print-patient").innerText = item.patientName;
    document.getElementById("print-pid").innerText = item.patientId;
    document.getElementById("print-appt-id").innerText = item.appointmentId;
    document.getElementById("print-invoice-id").innerText = item.invoiceId;
    document.getElementById("print-bill-date").innerText = item.billDateTime;
    document.getElementById("print-doctor").innerText = item.doctorName;

    // Consultation table body
    let consultRow = `
        <tr>
            <td>${item.consultDate}</td>
            <td>${item.consultDesc}</td>
            <td>1</td>
            <td class="text-end">${item.consultFee}</td>
        </tr>
        <tr>
            <td colspan="3" class="text-end"><strong>Subtotal</strong></td>
            <td class="text-end"><strong>${item.consultFee}</strong></td>
        </tr>
    `;
    document.getElementById("print-consult-tbody").innerHTML = consultRow;

    // Medicine table body
    let medRow = "";
    item.medicines.forEach(med => {
        medRow += `
            <tr>
                <td>${med.medDate}</td>
                <td>${med.name}</td>
                <td>${med.qty}</td>
                <td class="text-end">${med.price}</td>
            </tr>
        `;
    });
    medRow += `
        <tr>
            <td colspan="3" class="text-end"><strong>Subtotal</strong></td>
            <td class="text-end"><strong>${item.medicineSubtotal}</strong></td>
        </tr>
    `;
    document.getElementById("print-med-tbody").innerHTML = medRow;

    // Grand total
    document.getElementById("print-total").innerText = item.totalAmount;

    // Auto current print date time
    const now = new Date();
    const d = String(now.getDate()).padStart(2, '0');
    const m = String(now.getMonth() + 1).padStart(2, '0');
    const y = now.getFullYear();
    let h = now.getHours();
    const min = String(now.getMinutes()).padStart(2, '0');
    const ap = h >= 12 ? 'PM' : 'AM';
    h = h % 12 || 12;
    document.getElementById("print-now").innerText = `${d}/${m}/${y} ${h}:${min} ${ap}`;
}

function printBill() {
    window.print();
}
function printSingleBill(index) {
    const item = billingList[index];
    fillPrintTemplate(item);
    window.print();
}

function jumpToRecord(index) {
    const item = billingList[index];
    showToast(`Redirecting to Medical Record | ${item.appointmentId}`, "success");
    window.location.href = "Medical-Records.html";
}
function goToMedicalRecord() {
    if (currentViewIndex !== null) jumpToRecord(currentViewIndex);
}

document.addEventListener("DOMContentLoaded", () => {
    pagination.renderTable();
});