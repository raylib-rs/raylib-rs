// WS9 Phase 4 (Task 4.1): vanilla-JS name filter for the categorized gallery.
//
// Reads #filter, lowercases the query, and toggles the .hidden class on
// every .tile whose data-name doesn't contain the query. Empty categories
// are hidden as well so the page doesn't show bare <h2> headings with no
// tiles underneath.

(function () {
  const input = document.getElementById("filter");
  const status = document.getElementById("status");
  if (!input) return;

  const tiles = Array.from(document.querySelectorAll(".tile"));
  const categories = Array.from(document.querySelectorAll(".category"));
  const total = tiles.length;

  function apply() {
    const q = input.value.trim().toLowerCase();
    let visible = 0;
    for (const tile of tiles) {
      const name = (tile.dataset.name || "").toLowerCase();
      const match = q === "" || name.includes(q);
      tile.classList.toggle("hidden", !match);
      if (match) visible++;
    }
    for (const cat of categories) {
      const anyVisible = cat.querySelector(".tile:not(.hidden)") !== null;
      cat.classList.toggle("hidden", !anyVisible);
    }
    if (status) {
      if (q === "") {
        status.textContent = `${total} examples`;
      } else {
        status.textContent = `${visible} of ${total} examples match "${q}"`;
      }
    }
  }

  input.addEventListener("input", apply);
  apply();
})();
