// solver.ts
import init, {solve_day, init_panic_hook} from 'aoc-2024-wasm/aoc_2024_wasm';

interface PuzzleResult {
  part: number;
  result?: string;
  error?: string;
}

export class AocSolver {
  private initialized: boolean = false;

  async initialize(): Promise<void> {
    if (!this.initialized) {
      await init();
      init_panic_hook();
      this.initialized = true;
    }
  }

  async solvePuzzle(day: number, input: string): Promise<PuzzleResult[]> {
    await this.initialize();

    const parts = day === 25 ? [1] : [1, 2];
    return parts.map(part => {
      try {
        const response = solve_day(day, part, input);
        return {part, result: response.result};
      } catch (error: any) {
        return {part, error: error.toString()};
      }
    });
  }
}
