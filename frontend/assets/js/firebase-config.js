const firebaseConfig = {
    apiKey: "{{ firebase_api_key }}",
    authDomain: "{{ firebase_project_id }}.firebaseapp.com",
    projectId: "{{ firebase_project_id }}",
    storageBucket: "{{ firebase_project_id }}.appspot.com",
};
firebase.initializeApp(firebaseConfig);