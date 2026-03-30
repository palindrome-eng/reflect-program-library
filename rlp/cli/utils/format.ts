export function printSuccess(msg: string, data?: any) {
  console.log(`✓ ${msg}`);
  if (data) {
    console.log(JSON.stringify(data, bigintReplacer, 2));
  }
}

export function printError(err: unknown) {
  const msg = err instanceof Error ? err.message : String(err);
  console.error(`✗ ${msg}`);
  process.exit(1);
}

export function printTable(
  rows: Record<string, any>[],
  columns?: string[],
) {
  if (rows.length === 0) {
    console.log("(no results)");
    return;
  }
  const cols = columns ?? Object.keys(rows[0]);
  const widths = cols.map((c) =>
    Math.max(c.length, ...rows.map((r) => String(r[c] ?? "").length)),
  );
  const header = cols.map((c, i) => c.padEnd(widths[i])).join("  ");
  console.log(header);
  console.log(widths.map((w) => "-".repeat(w)).join("  "));
  for (const row of rows) {
    console.log(
      cols.map((c, i) => String(row[c] ?? "").padEnd(widths[i])).join("  "),
    );
  }
}

export function formatTokenAmount(raw: bigint, decimals: number): string {
  const str = raw.toString().padStart(decimals + 1, "0");
  const whole = str.slice(0, str.length - decimals);
  const frac = str.slice(str.length - decimals);
  return `${whole}.${frac}`;
}

function bigintReplacer(_key: string, value: any): any {
  return typeof value === "bigint" ? value.toString() : value;
}
