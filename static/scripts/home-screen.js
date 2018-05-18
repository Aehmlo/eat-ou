window.addEventListener("beforeinstallprompt", e => {
	e.preventDefault();
	deferredPrompt = e;
	let add = document.getElementById("add_button");
	add.style.display = "block";
	add.addEventListener("click", v => {
		add.style.display = 'none';
		e.prompt();
	}, { once: true });
});