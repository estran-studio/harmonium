// Initialiseur Trunk : page de démarrage pendant le téléchargement du wasm (étape 1 du chargement).
// `total` vaut 0 quand le serveur n'envoie pas `Content-Length` : la barre est alors indéterminée
// (un <progress> sans attribut `value`). L'écran egui prend le relais pour les ressources, et
// l'application retire #boot dès qu'elle tourne (platform/web.rs), parfois avant `onComplete` :
// chaque mise à jour vérifie que l'élément existe encore.
export default function initializer() {
  const bar = document.getElementById('boot-bar');
  const text = document.getElementById('boot-text');
  const megabytes = (bytes) => (bytes / 1e6).toFixed(1);
  const say = (message) => {
    if (text.isConnected) text.textContent = message;
  };

  return {
    onStart: () => say('Loading the application…'),
    onProgress: ({ current, total }) => {
      if (total > 0) {
        bar.max = total;
        bar.value = current;
        say(`Loading the application… ${megabytes(current)} / ${megabytes(total)} MB`);
      } else {
        bar.removeAttribute('value');
        say(`Loading the application… ${megabytes(current)} MB`);
      }
    },
    onComplete: () => {
      say('Starting…');
      bar.removeAttribute('value');
    },
    onSuccess: () => {},
    onFailure: (error) => {
      bar.style.display = 'none';
      say('The application could not be loaded.');
      const details = document.getElementById('boot-error');
      if (details) details.textContent = String(error);
    },
  };
}
