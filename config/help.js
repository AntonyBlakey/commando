const layout_horizontal = bindings => {
  for (let columns_per_row = Math.max(...bindings); columns_per_row > 0; columns_per_row--) {
    let row = 1;
    for (let group in bindings) {
      let column = 1;
      const start_row = row;
      for (let binding = 0; binding < bindings[group]; binding++) {
        if (column > columns_per_row) {
          column = 1;
          row++;
        }
        const key = document.getElementById(`key-${group}-${binding}`);
        key.style.setProperty("grid-row", String(row));
        key.style.setProperty("grid-column", String(2 * column));
        const label = document.getElementById(`label-${group}-${binding}`);
        label.style.setProperty("grid-row", String(row));
        label.style.setProperty("grid-column", String(2 * column + 1));
        if (row == start_row) {
          key.style.setProperty("margin-top", "0.2em");
          key.style.setProperty("padding-top", "0.2em");
          label.style.setProperty("margin-top", "0.2em");
          label.style.setProperty("padding-top", "0.2em");
        } else {
          key.style.setProperty("margin-top", "0");
          key.style.setProperty("padding-top", "0");
          label.style.setProperty("margin-top", "0");
          label.style.setProperty("padding-top", "0");
        }
        column++;
      }
      const group_label = document.getElementById(`group-label-${group}`);
      if (group_label) {
        group_label.style.setProperty("grid-row-start", String(start_row));
        group_label.style.setProperty("grid-row-end", String(row + 1)); // non-exclusive
        group_label.style.setProperty("grid-column", String(1));
        group_label.style.setProperty("margin-top", "0.2em");
        group_label.style.setProperty("padding-top", "0.2em");
        group_label.style.setProperty("padding-bottom", "0.2em");
      }
      const group_background = document.getElementById(`group-background-${group}`);
      if (group_background) {
        group_background.style.setProperty("grid-row-start", String(start_row));
        group_background.style.setProperty("grid-row-end", String(row + 1)); // non-exclusive
        group_background.style.setProperty("grid-column-start", String(1));
        group_background.style.setProperty("grid-column-end", String(2 * columns_per_row + 2));
        group_background.style.setProperty("margin-top", "0.2em");
      }
      row++;
    }
    if (document.getElementById("content").offsetWidth <= document.getElementById("body").offsetWidth) break;
  }
};

const layout_vertical = bindings => {
  for (let columns_per_row = Math.max(...bindings); columns_per_row > 0; columns_per_row--) {
    let row = 1;
    for (let group in bindings) {
      let column = 1;
      const start_row = row;
      for (let binding = 0; binding < bindings[group]; binding++) {
        if (column > columns_per_row) {
          column = 1;
          row++;
        }
        const key = document.getElementById(`key-${group}-${binding}`);
        key.style.setProperty("grid-row", String(row));
        key.style.setProperty("grid-column", String(2 * column));
        const label = document.getElementById(`label-${group}-${binding}`);
        label.style.setProperty("grid-row", String(row));
        label.style.setProperty("grid-column", String(2 * column + 1));
        if (row == start_row) {
          key.style.setProperty("margin-top", "0.2em");
          key.style.setProperty("padding-top", "0.2em");
          label.style.setProperty("margin-top", "0.2em");
          label.style.setProperty("padding-top", "0.2em");
        } else {
          key.style.setProperty("margin-top", "0");
          key.style.setProperty("padding-top", "0");
          label.style.setProperty("margin-top", "0");
          label.style.setProperty("padding-top", "0");
        }
        column++;
      }
      const group_label = document.getElementById(`group-label-${group}`);
      if (group_label) {
        group_label.style.setProperty("grid-row-start", String(start_row));
        group_label.style.setProperty("grid-row-end", String(row + 1)); // non-exclusive
        group_label.style.setProperty("grid-column", String(1));
        group_label.style.setProperty("margin-top", "0.2em");
        group_label.style.setProperty("padding-top", "0.2em");
        group_label.style.setProperty("padding-bottom", "0.2em");
      }
      const group_background = document.getElementById(`group-background-${group}`);
      if (group_background) {
        group_background.style.setProperty("grid-row-start", String(start_row));
        group_background.style.setProperty("grid-row-end", String(row + 1)); // non-exclusive
        group_background.style.setProperty("grid-column-start", String(1));
        group_background.style.setProperty("grid-column-end", String(2 * columns_per_row + 2));
        group_background.style.setProperty("margin-top", "0.2em");
      }
      row++;
    }
    if (document.getElementById("content").offsetWidth <= document.getElementById("body").offsetWidth) break;
  }
};

const layout = bindings => {
  layout_horizontal(bindings);
};
