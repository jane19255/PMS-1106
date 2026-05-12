const fb = new FirebaseService();

// SAVE PATIENT TO FIREBASE
async function savePatient() {
  const name = document.getElementById("name").value;
  const age = document.getElementById("age").value;

  await fb.addPatient({
    name: name,
    age: age
  });

  alert("Patient saved!");
  loadPatients();
}

// LOAD PATIENTS FROM FIREBASE
async function loadPatients() {
  const list = document.getElementById("patientList");
  list.innerHTML = "";

  const patients = await fb.getAllPatients();

  patients.forEach(p => {
    list.innerHTML += `<div class='card'>Name: ${p.name}, Age: ${p.age}</div>`;
  });
}

// AUTO LOAD WHEN PAGE OPEN
window.onload = loadPatients;