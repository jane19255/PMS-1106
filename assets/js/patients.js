const fb = new FirebaseService();

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

async function loadPatients() {
  const list = document.getElementById("patientList");
  list.innerHTML = "";

  const patients = await fb.getAllPatients();

  patients.forEach(p => {
    list.innerHTML += `<div class='card'>Name: ${p.name}, Age: ${p.age}</div>`;
  });
}

window.onload = loadPatients;