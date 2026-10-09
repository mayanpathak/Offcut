// The recognizer's model cache: the files the model manager put into OPFS,
// and nothing else (TS §16.4). The runtime asks its cache for a file before
// it would ask the network; this cache answers from OPFS or says no, and it
// never fetches. A file is in OPFS under its final name only after its hash
// matched the manifest's (INV-20), so a part file is never read here.

import { getFile, paths, size } from "../../persistence/opfs";

/** The two functions of the Web Cache API that the runtime calls. */
export type ModelCache = {
  match(request: string): Promise<Response>;
  put(request: string, response: Response): Promise<void>;
};

const PART_SUFFIX = ".part";

/**
 * A cache over the files of `modelId`. The runtime names a file by a path or
 * an address that ends in the file's name (`…/onnx/encoder_model_q4.onnx`);
 * the name is what the file is stored under (D-62). A file that is not
 * there is a failure, which the runtime reads as "not in the cache".
 */
export function createModelCache(modelId: string): ModelCache {
  return {
    async match(request) {
      const name = request.slice(request.lastIndexOf("/") + 1);
      if (name.endsWith(PART_SUFFIX)) {
        throw new Error("a part file is not a model file");
      }
      const path = paths.modelFile(modelId, name);
      const bytes = await size(path);
      if (bytes === null) {
        throw new Error("not a file of this model");
      }
      return new Response(await getFile(path), {
        headers: {
          "content-length": String(bytes),
          "content-type": name.endsWith(".json") ? "application/json" : "application/octet-stream",
        },
      });
    },
    // The runtime offers what it loaded for keeping. The files are kept already.
    put: () => Promise.resolve(),
  };
}
