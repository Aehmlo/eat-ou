const CACHE_NAME = "eat-ou-v1";
const urls = [
  "/",
  "/css/style.css",
  "/eat-ou.js",
  "/eat-ou.wasm"
];

self.addEventListener("install", e => {
  e.waitUntil(caches.open(CACHE_NAME).then(cache => cache.addAll(urls)));
});

self.addEventListener("fetch", e => {
  e.respondWith(caches.match(e.request).then(r => (r || fetch(e.request))));
})
