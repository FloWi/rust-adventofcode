// types.ts
export interface WasmModule {
  solve_day: (day: number, part: number, input: string) => SolutionResult;
  init_panic_hook: () => void;
  memory: WebAssembly.Memory;
}

export interface SolutionResult {
  result: string;
}

// Using the exact name from the generated .d.ts file
declare module 'aoc-2024-wasm/aoc_2024_wasm' {
  export function solve_day(day: number, part: number, input: string): SolutionResult;

  export function init_panic_hook(): void;

  export default function init(): Promise<void>;
}
