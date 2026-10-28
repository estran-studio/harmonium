// Initialiseur Trunk : page de démarrage pendant le téléchargement du wasm (étape 1 du chargement).
// `total` vaut 0 quand le serveur n'envoie pas `Content-Length` : la barre est alors indéterminée
// (un <progress> sans attribut `value`). L'écran egui prend le relais pour les ressources, et
// l'application retire #boot dès qu'elle tourne (platform/web.rs).
export default function initializer() {
  const bar = () => document.getElementById('boot-bar');
  const text = () => document.getElementById('boot-text');
  const megabytes = (bytes) => (bytes / 1e6).toFixed(1);

  return {
    onStart: () => {
      text().textContent = 'Loading the application…';
    },
    onProgress: ({ current, total }) => {
      if (total > 0) {
        bar().max = total;
        bar().value = current;
        text().textContent = `Loading the application… ${megabytes(current)} / ${megabytes(total)} MB`;
      } else {
        bar().removeAttribute('value');
        text().textContent = `Loading the application… ${megabytes(current)} MB`;
      }
    },
    onComplete: () => {
      text().textContent = 'Starting…';
      bar().removeAttribute('value');
    },
    onSuccess: () => {},
    onFailure: (error) => {
      bar().style.display = 'none';
      text().textContent = 'The application could not be loaded.';
      document.getElementById('boot-error').textContent = String(error);
    },
  };
}
