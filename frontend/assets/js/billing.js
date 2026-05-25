(function () {
  const STORAGE_KEY = "pms_invoices";

  const invoiceForm = document.getElementById("invoiceForm");
  const patientIdInput = document.getElementById("patientId");
  const treatmentNameInput = document.getElementById("treatmentName");
  const treatmentCostInput = document.getElementById("treatmentCost");
  const prescriptionNameInput = document.getElementById("prescriptionName");
  const prescriptionCostInput = document.getElementById("prescriptionCost");
  const invoiceList = document.getElementById("invoiceList");
  const reportContent = document.getElementById("reportContent");
  const printReportButton = document.getElementById("printReportButton");

  function getInvoices() {
    return JSON.parse(localStorage.getItem(STORAGE_KEY)) || [];
  }

  function saveInvoices(invoices) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(invoices));
  }

  function money(value) {
    return `$${Number(value).toFixed(2)}`;
  }

  function createInvoice(event) {
    event.preventDefault();

    const treatmentCost = Number(treatmentCostInput.value);
    const prescriptionCost = Number(prescriptionCostInput.value);

    if (!patientIdInput.value.trim() || !treatmentNameInput.value.trim()) {
      alert("Please enter a patient ID and treatment name.");
      return;
    }

    if (treatmentCost < 0 || prescriptionCost < 0) {
      alert("Costs cannot be negative.");
      return;
    }

    const items = [
      {
        type: "Treatment",
        name: treatmentNameInput.value.trim(),
        cost: treatmentCost || 0
      }
    ];

    if (prescriptionNameInput.value.trim()) {
      items.push({
        type: "Prescription",
        name: prescriptionNameInput.value.trim(),
        cost: prescriptionCost || 0
      });
    }

    const subtotal = items.reduce((total, item) => total + item.cost, 0);
    const invoice = {
      id: `INV-${Date.now()}`,
      patientId: patientIdInput.value.trim(),
      items,
      subtotal,
      total: subtotal,
      status: "Pending",
      createdAt: new Date().toLocaleString()
    };

    const invoices = getInvoices();
    invoices.unshift(invoice);
    saveInvoices(invoices);

    invoiceForm.reset();
    renderInvoices();
    renderReport(invoice);
  }

  function markAsPaid(invoiceId) {
    const invoices = getInvoices().map((invoice) => {
      if (invoice.id !== invoiceId) {
        return invoice;
      }

      return {
        ...invoice,
        status: "Paid"
      };
    });

    saveInvoices(invoices);
    renderInvoices();

    const paidInvoice = invoices.find((invoice) => invoice.id === invoiceId);
    renderReport(paidInvoice);
  }

  function renderInvoices() {
    const invoices = getInvoices();
    invoiceList.innerHTML = "";

    if (invoices.length === 0) {
      invoiceList.textContent = "No invoices created yet.";
      return;
    }

    invoices.forEach((invoice) => {
      const invoiceCard = document.createElement("div");
      invoiceCard.className = "card invoice-card";

      const itemList = invoice.items
        .map((item) => `<li>${item.type}: ${item.name} - ${money(item.cost)}</li>`)
        .join("");

      invoiceCard.innerHTML = `
        <h3>${invoice.id}</h3>
        <p>Patient ID: ${invoice.patientId}</p>
        <p>Created: ${invoice.createdAt}</p>
        <ul>${itemList}</ul>
        <p>Subtotal: ${money(invoice.subtotal)}</p>
        <p>Total: ${money(invoice.total)}</p>
        <p>Status: ${invoice.status}</p>
      `;

      if (invoice.status !== "Paid") {
        const payButton = document.createElement("button");
        payButton.type = "button";
        payButton.textContent = "Mark as Paid";
        payButton.addEventListener("click", () => markAsPaid(invoice.id));
        invoiceCard.appendChild(payButton);
      }

      invoiceCard.addEventListener("click", () => renderReport(invoice));
      invoiceList.appendChild(invoiceCard);
    });
  }

  function renderReport(invoice) {
    if (!invoice) {
      return;
    }

    const reportItems = invoice.items
      .map((item) => `<li>${item.type}: ${item.name} - ${money(item.cost)}</li>`)
      .join("");

    reportContent.innerHTML = `
      <p>Invoice: ${invoice.id}</p>
      <p>Patient ID: ${invoice.patientId}</p>
      <p>Date: ${invoice.createdAt}</p>
      <ul>${reportItems}</ul>
      <p>Total: ${money(invoice.total)}</p>
      <p>Status: ${invoice.status}</p>
    `;
  }

  invoiceForm.addEventListener("submit", createInvoice);
  printReportButton.addEventListener("click", () => window.print());
  renderInvoices();
})();
