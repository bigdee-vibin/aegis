/* Theme toggle. Dark-first; the choice is kept in localStorage only if available. No network. */
(function () {
  var root = document.documentElement, KEY = 'aegis-site-theme', saved = null;
  try { saved = localStorage.getItem(KEY); } catch (e) {}
  root.setAttribute('data-theme', saved === 'light' ? 'light' : 'dark');
  document.addEventListener('DOMContentLoaded', function () {
    var b = document.getElementById('theme-toggle');
    if (!b) return;
    function paint() {
      var light = root.getAttribute('data-theme') === 'light';
      b.textContent = light ? 'Dark theme' : 'Light theme';
      b.setAttribute('aria-label', light ? 'Switch to dark theme' : 'Switch to light theme');
    }
    paint();
    b.addEventListener('click', function () {
      var next = root.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
      root.setAttribute('data-theme', next);
      try { localStorage.setItem(KEY, next); } catch (e) {}
      paint();
    });
  });
})();
