export async function loadGuideJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`指南资源加载失败：${response.status}`);
  const decoded = url.endsWith(".gz")
    ? new Response(response.body!.pipeThrough(new DecompressionStream("gzip")))
    : response;
  return decoded.json() as Promise<T>;
}
