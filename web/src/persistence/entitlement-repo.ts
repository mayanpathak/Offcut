// The entitlement token this device holds (TS §23.1): one record, under the
// key "current". The token is stored as it came and is never read here: the
// render worker is what verifies it (INV-9).

import { openDb } from "./db";
import { DB_VERSION, type EntitlementRecord, STORES, type Versioned } from "./schema";

const KEY = "current";

/** The stored token and when it was stored, or `undefined` if there is none. */
export async function get(): Promise<EntitlementRecord | undefined> {
  const db = await openDb();
  const stored = await db.get(STORES.entitlement, KEY);
  if (stored === undefined) {
    return undefined;
  }
  if (stored.schemaVersion > DB_VERSION) {
    // Written by a newer build: absent to this one, and removed (TS §23.3).
    await db.delete(STORES.entitlement, KEY);
    return undefined;
  }
  return stored.value;
}

export async function put(token: string): Promise<void> {
  const db = await openDb();
  const stored: Versioned<EntitlementRecord> = { schemaVersion: DB_VERSION, value: { token, storedAt: Date.now() } };
  await db.put(STORES.entitlement, stored, KEY);
}
