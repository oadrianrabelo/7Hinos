// Desenha as páginas de um PDF como imagens (pdf.js). Carregado só quando
// alguém importa um PDF, para não pesar na abertura do app nem nas telas.

/** Largura das páginas desenhadas: nítida num telão Full HD. */
const WIDTH = 1920;

export async function renderPdf(
  data: ArrayBuffer,
  onPage: (jpeg: Uint8Array, page: number, total: number) => Promise<void>,
): Promise<number> {
  const pdfjs = await import("pdfjs-dist");
  const worker = (await import("pdfjs-dist/build/pdf.worker.min.mjs?url")).default;
  pdfjs.GlobalWorkerOptions.workerSrc = worker;

  const doc = await pdfjs.getDocument({ data: new Uint8Array(data) }).promise;
  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Este computador não conseguiu desenhar o PDF.");
  try {
    for (let n = 1; n <= doc.numPages; n++) {
      const page = await doc.getPage(n);
      const base = page.getViewport({ scale: 1 });
      // O lado maior com 1920 px: nítido no telão, sem arquivos enormes
      // (páginas em pé, como a lição, ficam com 1920 de altura).
      const scale = WIDTH / Math.max(base.width, base.height);
      const viewport = page.getViewport({ scale });
      canvas.width = Math.round(viewport.width);
      canvas.height = Math.round(viewport.height);
      context.fillStyle = "#fff";
      context.fillRect(0, 0, canvas.width, canvas.height);
      await page.render({ canvasContext: context, viewport }).promise;
      const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, "image/jpeg", 0.9));
      if (!blob) throw new Error(`Não deu para converter a página ${n}.`);
      await onPage(new Uint8Array(await blob.arrayBuffer()), n, doc.numPages);
      page.cleanup();
    }
    return doc.numPages;
  } finally {
    await doc.destroy();
  }
}
