// PDF Toolkit Rust Client Controller
document.addEventListener('DOMContentLoaded', () => {
  initTheme();
  initToolCards();
  initModal();
});

// PDF.js worker setup
if (window.pdfjsLib) {
  pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';
}

function initTheme() {
  const toggleBtn = document.getElementById('themeToggleBtn');
  const themeIcon = toggleBtn.querySelector('.theme-icon');
  const themeLabel = toggleBtn.querySelector('.theme-label');
  const htmlEl = document.documentElement;

  const savedTheme = localStorage.getItem('rust_pdf_theme') || 'dark';
  applyTheme(savedTheme);

  toggleBtn.addEventListener('click', () => {
    const currentTheme = htmlEl.getAttribute('data-theme');
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    applyTheme(newTheme);
  });

  function applyTheme(theme) {
    htmlEl.setAttribute('data-theme', theme);
    localStorage.setItem('rust_pdf_theme', theme);
    if (theme === 'dark') {
      themeIcon.textContent = '☀️';
      themeLabel.textContent = 'Light Mode';
    } else {
      themeIcon.textContent = '🌙';
      themeLabel.textContent = 'Dark Mode';
    }
  }
}

const modal = document.getElementById('toolModal');
const modalTitle = document.getElementById('modalTitle');
const modalIcon = document.getElementById('modalIcon');
const modalBody = document.getElementById('modalBody');
const modalCloseBtn = document.getElementById('modalCloseBtn');

function initModal() {
  modalCloseBtn.addEventListener('click', closeModal);
  modal.addEventListener('click', (e) => {
    if (e.target === modal) closeModal();
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !modal.classList.contains('hidden')) {
      closeModal();
    }
  });
}

function openModal(title, icon, contentHtml, onMounted) {
  modalTitle.textContent = title;
  modalIcon.textContent = icon;
  modalBody.innerHTML = contentHtml;
  modal.classList.remove('hidden');
  if (onMounted) onMounted();
}

function closeModal() {
  modal.classList.add('hidden');
  modalBody.innerHTML = '';
}

function showToast(message, type = 'info') {
  const toast = document.getElementById('toast');
  toast.textContent = message;
  toast.className = `toast toast-${type}`;
  toast.classList.remove('hidden');
  setTimeout(() => {
    toast.classList.add('hidden');
  }, 4000);
}

function initToolCards() {
  document.querySelectorAll('.tool-card').forEach(card => {
    card.addEventListener('click', () => {
      const tool = card.getAttribute('data-tool');
      routeTool(tool);
    });
  });
}

function routeTool(tool) {
  switch (tool) {
    case 'merge': setupMergeTool(); break;
    case 'split': setupSplitTool(); break;
    case 'organize': setupOrganizeTool(); break;
    case 'compress': setupCompressTool(); break;
    case 'images-to-pdf': setupImagesToPdfTool(); break;
    case 'watermark': setupWatermarkTool(); break;
    case 'page-numbers': setupPageNumbersTool(); break;
    case 'extract-text': setupExtractTextTool(); break;
    default: showToast('Werkzeug in Vorbereitung', 'info');
  }
}

function setupDropzone(dropzoneEl, fileInputEl, onFilesSelected) {
  dropzoneEl.addEventListener('click', () => fileInputEl.click());
  fileInputEl.addEventListener('change', (e) => {
    if (e.target.files && e.target.files.length > 0) {
      onFilesSelected(Array.from(e.target.files));
    }
  });

  ['dragenter', 'dragover'].forEach(eventName => {
    dropzoneEl.addEventListener(eventName, (e) => {
      e.preventDefault();
      dropzoneEl.classList.add('dragover');
    }, false);
  });

  ['dragleave', 'drop'].forEach(eventName => {
    dropzoneEl.addEventListener(eventName, (e) => {
      e.preventDefault();
      dropzoneEl.classList.remove('dragover');
    }, false);
  });

  dropzoneEl.addEventListener('drop', (e) => {
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      onFilesSelected(Array.from(e.dataTransfer.files));
    }
  });
}

function fileToBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const b64 = reader.result.split(',')[1];
      resolve(b64);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

function downloadBase64(b64Data, filename, mimeType = 'application/pdf') {
  const byteChars = atob(b64Data);
  const byteNumbers = new Array(byteChars.length);
  for (let i = 0; i < byteChars.length; i++) {
    byteNumbers[i] = byteChars.charCodeAt(i);
  }
  const byteArray = new Uint8Array(byteNumbers);
  const blob = new Blob([byteArray], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.style.display = 'none';
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  URL.revokeObjectURL(url);
  document.body.removeChild(a);
}

// 1. MERGE
function setupMergeTool() {
  const html = `
    <div class="dropzone" id="mergeDropzone">
      <div class="dropzone-icon">📑</div>
      <h4>PDF-Dateien hier ablegen</h4>
      <p>oder klicken zum Auswählen</p>
      <input type="file" id="mergeFileInput" accept=".pdf" multiple style="display:none">
    </div>
    <ul class="file-list" id="mergeFileList"></ul>
    <div style="display:flex; justify-content:flex-end; gap:0.5rem;">
      <button class="btn btn-primary" id="mergeSubmitBtn" disabled>Zusammenfügen & Herunterladen</button>
    </div>
  `;

  openModal('PDF zusammenfügen', '📑', html, () => {
    let files = [];
    const listEl = document.getElementById('mergeFileList');
    const submitBtn = document.getElementById('mergeSubmitBtn');
    const dropzone = document.getElementById('mergeDropzone');
    const fileInput = document.getElementById('mergeFileInput');

    function render() {
      listEl.innerHTML = '';
      files.forEach((f, idx) => {
        const li = document.createElement('li');
        li.className = 'file-item';
        li.innerHTML = `
          <span><strong>${idx + 1}.</strong> ${f.name}</span>
          <div class="file-item-actions">
            <button class="page-btn move-up" data-idx="${idx}" ${idx === 0 ? 'disabled' : ''}>▲</button>
            <button class="page-btn move-down" data-idx="${idx}" ${idx === files.length - 1 ? 'disabled' : ''}>▼</button>
            <button class="page-btn del-btn" data-idx="${idx}">✕</button>
          </div>
        `;
        listEl.appendChild(li);
      });
      submitBtn.disabled = files.length < 2;

      listEl.querySelectorAll('.move-up').forEach(b => b.onclick = () => {
        const i = +b.dataset.idx;
        const temp = files[i]; files[i] = files[i-1]; files[i-1] = temp; render();
      });
      listEl.querySelectorAll('.move-down').forEach(b => b.onclick = () => {
        const i = +b.dataset.idx;
        const temp = files[i]; files[i] = files[i+1]; files[i+1] = temp; render();
      });
      listEl.querySelectorAll('.del-btn').forEach(b => b.onclick = () => {
        files.splice(+b.dataset.idx, 1); render();
      });
    }

    setupDropzone(dropzone, fileInput, (newFiles) => {
      files = files.concat(newFiles);
      render();
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Zusammenfügen...';
      try {
        const base64Files = await Promise.all(files.map(fileToBase64));
        const res = await fetch('/api/merge', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ files: base64Files })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, 'zusammengefuegt.pdf');
        showToast('PDFs erfolgreich zusammengefügt!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Zusammenfügen & Herunterladen';
      }
    };
  });
}

// 2. SPLIT
function setupSplitTool() {
  const html = `
    <div class="dropzone" id="splitDropzone">
      <div class="dropzone-icon">✂️</div>
      <h4>PDF hier ablegen</h4>
      <p id="splitFileName">oder klicken zum Auswählen</p>
      <input type="file" id="splitFileInput" accept=".pdf" style="display:none">
    </div>
    <div id="splitOptions" class="hidden">
      <div class="form-group">
        <label>Modus wählen:</label>
        <div style="display:flex; gap:1.5rem; margin-top:0.5rem;">
          <label style="cursor:pointer;"><input type="radio" name="splitMode" value="range" checked> Seitenbereich</label>
          <label style="cursor:pointer;"><input type="radio" name="splitMode" value="single"> Jede Seite einzeln (ZIP)</label>
        </div>
      </div>
      <div class="form-group" id="rangeGroup">
        <label>Seiten (z. B. "1-3, 5"):</label>
        <input type="text" id="splitRange" class="form-control" placeholder="1-3, 5">
      </div>
      <div style="display:flex; justify-content:flex-end;">
        <button class="btn btn-primary" id="splitSubmitBtn">Teilen & Herunterladen</button>
      </div>
    </div>
  `;

  openModal('PDF teilen', '✂️', html, () => {
    let file = null;
    const dropzone = document.getElementById('splitDropzone');
    const fileInput = document.getElementById('splitFileInput');
    const options = document.getElementById('splitOptions');
    const fileName = document.getElementById('splitFileName');
    const rangeGroup = document.getElementById('rangeGroup');
    const submitBtn = document.getElementById('splitSubmitBtn');

    setupDropzone(dropzone, fileInput, (files) => {
      file = files[0];
      fileName.textContent = file.name;
      options.classList.remove('hidden');
    });

    document.querySelectorAll('input[name="splitMode"]').forEach(r => {
      r.onchange = (e) => {
        rangeGroup.classList.toggle('hidden', e.target.value === 'single');
      };
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Verarbeite...';
      try {
        const b64 = await fileToBase64(file);
        const mode = document.querySelector('input[name="splitMode"]:checked').value;
        const range = document.getElementById('splitRange').value;

        const res = await fetch('/api/split', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: b64, mode, range })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, data.filename, mode === 'single' ? 'application/zip' : 'application/pdf');
        showToast('PDF erfolgreich aufgeteilt!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Teilen & Herunterladen';
      }
    };
  });
}

// 3. ORGANIZE & ROTATE (Client-side PDF.js thumbnail generation)
function setupOrganizeTool() {
  const html = `
    <div class="dropzone" id="orgDropzone">
      <div class="dropzone-icon">🔄</div>
      <h4>PDF hier ablegen für visuelle Vorschau</h4>
      <p id="orgFileName">oder klicken zum Auswählen</p>
      <input type="file" id="orgFileInput" accept=".pdf" style="display:none">
    </div>
    <div id="orgArea" class="hidden">
      <p style="font-size:0.85rem; color:var(--text-muted); margin-bottom:0.75rem;">
        Drehe Seiten mit ↻, ordne sie mit ◀ ▶ oder lösche Seiten mit ✕.
      </p>
      <div class="pages-grid" id="pagesGrid"></div>
      <div style="display:flex; justify-content:flex-end;">
        <button class="btn btn-primary" id="orgSubmitBtn">Änderungen speichern & Herunterladen</button>
      </div>
    </div>
  `;

  openModal('Seiten verwalten & drehen', '🔄', html, () => {
    let file = null;
    let pages = []; // [{ pageNum: 1, rotation: 0, canvas: ... }]
    const dropzone = document.getElementById('orgDropzone');
    const fileInput = document.getElementById('orgFileInput');
    const orgArea = document.getElementById('orgArea');
    const grid = document.getElementById('pagesGrid');
    const submitBtn = document.getElementById('orgSubmitBtn');

    function renderGrid() {
      grid.innerHTML = '';
      pages.forEach((p, idx) => {
        const card = document.createElement('div');
        card.className = 'page-card';
        card.innerHTML = `
          <div class="page-thumb-wrap" id="thumb-wrap-${idx}"></div>
          <div class="page-controls">
            <button class="page-btn move-left" data-idx="${idx}" ${idx === 0 ? 'disabled' : ''}>◀</button>
            <span class="page-num">S. ${p.pageNum}</span>
            <button class="page-btn rotate-btn" data-idx="${idx}">↻</button>
            <button class="page-btn move-right" data-idx="${idx}" ${idx === pages.length - 1 ? 'disabled' : ''}>▶</button>
            <button class="page-btn del-btn" data-idx="${idx}">✕</button>
          </div>
        `;
        grid.appendChild(card);
        const wrap = card.querySelector(`#thumb-wrap-${idx}`);
        p.canvas.style.transform = `rotate(${p.rotation}deg)`;
        wrap.appendChild(p.canvas);
      });

      grid.querySelectorAll('.rotate-btn').forEach(b => b.onclick = () => {
        const i = +b.dataset.idx;
        pages[i].rotation = (pages[i].rotation + 90) % 360;
        pages[i].canvas.style.transform = `rotate(${pages[i].rotation}deg)`;
      });
      grid.querySelectorAll('.move-left').forEach(b => b.onclick = () => {
        const i = +b.dataset.idx;
        const temp = pages[i]; pages[i] = pages[i-1]; pages[i-1] = temp; renderGrid();
      });
      grid.querySelectorAll('.move-right').forEach(b => b.onclick = () => {
        const i = +b.dataset.idx;
        const temp = pages[i]; pages[i] = pages[i+1]; pages[i+1] = temp; renderGrid();
      });
      grid.querySelectorAll('.del-btn').forEach(b => b.onclick = () => {
        pages.splice(+b.dataset.idx, 1); renderGrid();
      });
    }

    setupDropzone(dropzone, fileInput, async (files) => {
      file = files[0];
      dropzone.innerHTML = '<span class="spinner"></span> Rendere Vorschau in Rust/Browser...';

      try {
        const arrayBuf = await file.arrayBuffer();
        const pdf = await pdfjsLib.getDocument({ data: arrayBuf }).promise;
        pages = [];

        for (let i = 1; i <= pdf.numPages; i++) {
          const page = await pdf.getPage(i);
          const viewport = page.getViewport({ scale: 0.25 });
          const canvas = document.createElement('canvas');
          const context = canvas.getContext('2d');
          canvas.height = viewport.height;
          canvas.width = viewport.width;

          await page.render({ canvasContext: context, viewport: viewport }).promise;
          pages.push({ pageNum: i, rotation: 0, canvas });
        }

        dropzone.classList.add('hidden');
        orgArea.classList.remove('hidden');
        renderGrid();
      } catch (err) {
        showToast('Fehler beim Laden: ' + err.message, 'error');
      }
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Speichere...';
      try {
        const b64 = await fileToBase64(file);
        const order = pages.map(p => p.pageNum);
        const rotations = pages.map(p => [p.pageNum, p.rotation]);

        const res = await fetch('/api/organize', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: b64, order, rotations })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, 'organisiert.pdf');
        showToast('PDF erfolgreich gespeichert!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Änderungen speichern & Herunterladen';
      }
    };
  });
}

// 4. COMPRESS
function setupCompressTool() {
  const html = `
    <div class="dropzone" id="compDropzone">
      <div class="dropzone-icon">🗜️</div>
      <h4>PDF zum Optimieren ablegen</h4>
      <p id="compFileName">oder klicken zum Auswählen</p>
      <input type="file" id="compFileInput" accept=".pdf" style="display:none">
    </div>
    <div id="compOptions" class="hidden" style="display:flex; justify-content:flex-end;">
      <button class="btn btn-primary" id="compSubmitBtn">In Rust komprimieren</button>
    </div>
  `;

  openModal('PDF komprimieren', '🗜️', html, () => {
    let file = null;
    const dropzone = document.getElementById('compDropzone');
    const fileInput = document.getElementById('compFileInput');
    const options = document.getElementById('compOptions');
    const fileName = document.getElementById('compFileName');
    const submitBtn = document.getElementById('compSubmitBtn');

    setupDropzone(dropzone, fileInput, (files) => {
      file = files[0];
      fileName.textContent = `${file.name} (${(file.size/1024/1024).toFixed(2)} MB)`;
      options.classList.remove('hidden');
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Komprimiere...';
      try {
        const b64 = await fileToBase64(file);
        const res = await fetch('/api/compress', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: b64 })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, 'komprimiert.pdf');
        const origMb = (data.orig_size / 1024 / 1024).toFixed(2);
        const newMb = (data.new_size / 1024 / 1024).toFixed(2);
        const percent = ((1 - data.new_size / data.orig_size) * 100).toFixed(1);
        showToast(`Komprimiert: ${origMb}MB -> ${newMb}MB (-${percent}%)`, 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'In Rust komprimieren';
      }
    };
  });
}

// 5. IMAGES TO PDF
function setupImagesToPdfTool() {
  const html = `
    <div class="dropzone" id="imgDropzone">
      <div class="dropzone-icon">🖼️</div>
      <h4>Bilder hier ablegen (JPEG/JPG)</h4>
      <p id="imgCountText">oder klicken zum Auswählen</p>
      <input type="file" id="imgFileInput" accept="image/jpeg,image/jpg" multiple style="display:none">
    </div>
    <div id="imgOptions" class="hidden" style="display:flex; justify-content:flex-end;">
      <button class="btn btn-primary" id="imgSubmitBtn">In PDF konvertieren</button>
    </div>
  `;

  openModal('Bilder zu PDF', '🖼️', html, () => {
    let files = [];
    const dropzone = document.getElementById('imgDropzone');
    const fileInput = document.getElementById('imgFileInput');
    const options = document.getElementById('imgOptions');
    const countText = document.getElementById('imgCountText');
    const submitBtn = document.getElementById('imgSubmitBtn');

    setupDropzone(dropzone, fileInput, (newFiles) => {
      files = files.concat(newFiles);
      countText.textContent = `${files.length} Bild(er) ausgewählt`;
      options.classList.remove('hidden');
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Erstelle PDF...';
      try {
        const b64Files = await Promise.all(files.map(fileToBase64));
        const res = await fetch('/api/images-to-pdf', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ images: b64Files })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, 'bilder.pdf');
        showToast('PDF erfolgreich erstellt!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'In PDF konvertieren';
      }
    };
  });
}

// 6. WATERMARK
function setupWatermarkTool() {
  const html = `
    <div class="dropzone" id="wmDropzone">
      <div class="dropzone-icon">💧</div>
      <h4>PDF hier ablegen</h4>
      <p id="wmFileName">oder klicken zum Auswählen</p>
      <input type="file" id="wmFileInput" accept=".pdf" style="display:none">
    </div>
    <div id="wmOptions" class="hidden">
      <div class="form-group">
        <label>Wasserzeichen-Text:</label>
        <input type="text" id="wmText" class="form-control" value="VERTRAULICH">
      </div>
      <div style="display:flex; justify-content:flex-end;">
        <button class="btn btn-primary" id="wmSubmitBtn">Wasserzeichen anwenden</button>
      </div>
    </div>
  `;

  openModal('Wasserzeichen hinzufügen', '💧', html, () => {
    let file = null;
    const dropzone = document.getElementById('wmDropzone');
    const fileInput = document.getElementById('wmFileInput');
    const options = document.getElementById('wmOptions');
    const fileName = document.getElementById('wmFileName');
    const submitBtn = document.getElementById('wmSubmitBtn');

    setupDropzone(dropzone, fileInput, (files) => {
      file = files[0];
      fileName.textContent = file.name;
      options.classList.remove('hidden');
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Wende an...';
      try {
        const b64 = await fileToBase64(file);
        const text = document.getElementById('wmText').value;
        const res = await fetch('/api/watermark', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: b64, text })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, 'wasserzeichen.pdf');
        showToast('Wasserzeichen angewendet!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Wasserzeichen anwenden';
      }
    };
  });
}

// 7. PAGE NUMBERS
function setupPageNumbersTool() {
  const html = `
    <div class="dropzone" id="numDropzone">
      <div class="dropzone-icon">🔢</div>
      <h4>PDF hier ablegen</h4>
      <p id="numFileName">oder klicken zum Auswählen</p>
      <input type="file" id="numFileInput" accept=".pdf" style="display:none">
    </div>
    <div id="numOptions" class="hidden">
      <div class="form-group">
        <label>Position:</label>
        <select id="numPosition" class="form-control">
          <option value="bottom-center">Unten Mitte</option>
          <option value="bottom-right">Unten Rechts</option>
          <option value="bottom-left">Unten Links</option>
          <option value="top-center">Oben Mitte</option>
          <option value="top-right">Oben Rechts</option>
        </select>
      </div>
      <div class="form-group">
        <label>Format:</label>
        <select id="numFormat" class="form-control">
          <option value="seite_x_von_y">Seite X von Y</option>
          <option value="x_von_y">X / Y</option>
          <option value="x">Nur Nummer (X)</option>
        </select>
      </div>
      <div style="display:flex; justify-content:flex-end;">
        <button class="btn btn-primary" id="numSubmitBtn">Seitenzahlen einfügen</button>
      </div>
    </div>
  `;

  openModal('Seitenzahlen hinzufügen', '🔢', html, () => {
    let file = null;
    const dropzone = document.getElementById('numDropzone');
    const fileInput = document.getElementById('numFileInput');
    const options = document.getElementById('numOptions');
    const fileName = document.getElementById('numFileName');
    const submitBtn = document.getElementById('numSubmitBtn');

    setupDropzone(dropzone, fileInput, (files) => {
      file = files[0];
      fileName.textContent = file.name;
      options.classList.remove('hidden');
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Nummeriere...';
      try {
        const b64 = await fileToBase64(file);
        const position = document.getElementById('numPosition').value;
        const format = document.getElementById('numFormat').value;

        const res = await fetch('/api/page-numbers', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: b64, position, format })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        downloadBase64(data.data, 'nummeriert.pdf');
        showToast('Seitenzahlen eingefügt!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Seitenzahlen einfügen';
      }
    };
  });
}

// 8. EXTRACT TEXT
function setupExtractTextTool() {
  const html = `
    <div class="dropzone" id="txtDropzone">
      <div class="dropzone-icon">📝</div>
      <h4>PDF hier ablegen</h4>
      <p id="txtFileName">oder klicken zum Auswählen</p>
      <input type="file" id="txtFileInput" accept=".pdf" style="display:none">
    </div>
    <div id="txtOptions" class="hidden" style="display:flex; justify-content:flex-end;">
      <button class="btn btn-primary" id="txtSubmitBtn">Text als .txt herunterladen</button>
    </div>
  `;

  openModal('Text extrahieren', '📝', html, () => {
    let file = null;
    const dropzone = document.getElementById('txtDropzone');
    const fileInput = document.getElementById('txtFileInput');
    const options = document.getElementById('txtOptions');
    const fileName = document.getElementById('txtFileName');
    const submitBtn = document.getElementById('txtSubmitBtn');

    setupDropzone(dropzone, fileInput, (files) => {
      file = files[0];
      fileName.textContent = file.name;
      options.classList.remove('hidden');
    });

    submitBtn.onclick = async () => {
      submitBtn.disabled = true;
      submitBtn.innerHTML = '<span class="spinner"></span>Extrahiere...';
      try {
        const b64 = await fileToBase64(file);
        const res = await fetch('/api/extract-text', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: b64 })
        });
        const data = await res.json();
        if (!data.ok) throw new Error(data.error);

        // Download raw text
        const blob = new Blob([data.text], { type: 'text/plain;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'extrahierter_text.txt';
        document.body.appendChild(a);
        a.click();
        URL.revokeObjectURL(url);
        document.body.removeChild(a);

        showToast('Text erfolgreich extrahiert!', 'success');
        closeModal();
      } catch (err) {
        showToast(err.message, 'error');
      } finally {
        submitBtn.disabled = false;
        submitBtn.textContent = 'Text als .txt herunterladen';
      }
    };
  });
}
