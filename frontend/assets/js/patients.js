(function () {
  const patientService = new window.PatientBackend.FirebaseService();

  const patientForm = document.getElementById("patientForm");
  const nameInput = document.getElementById("name");
  const ageInput = document.getElementById("age");
  const patientList = document.getElementById("patientList");

  async function savePatient(event) {
    event.preventDefault();

    const patient = {
      name: nameInput.value.trim(),
      age: ageInput.value.trim()
    };

    if (!patient.name || !patient.age) {
      alert("Please enter both patient name and age.");
      return;
    }

    await patientService.addPatient(patient);
    patientForm.reset();
    await loadPatients();
  }

  async function loadPatients() {
    patientList.innerHTML = "";

    const patients = await patientService.getAllPatients();

    patients.forEach((patient) => {
      const patientCard = document.createElement("div");
      patientCard.className = "card";
      patientCard.textContent = `Name: ${patient.name}, Age: ${patient.age}`;
      patientList.appendChild(patientCard);
    });
  }

  patientForm.addEventListener("submit", savePatient);
  window.addEventListener("load", loadPatients);
})();
