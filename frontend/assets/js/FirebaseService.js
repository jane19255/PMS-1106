class FirebaseService {
  constructor() {
    this.db = firebase.firestore();
  }

  async addPatient(patient) {
    return await this.db.collection("patients").add(patient);
  }

  async getAllPatients() {
    const snap = await this.db.collection("patients").get();
    return snap.docs.map(doc => ({ id: doc.id, ...doc.data() }));
  }
}