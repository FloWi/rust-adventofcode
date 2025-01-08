// solver.ts
// Your imports should now look like this
import init, {get_testcases, init_panic_hook, Part, solve_day} from '../wasm/aoc_2024_wasm';

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

  async printTestcases() {
    await this.initialize();
    get_testcases().forEach((tc) => {
      console.log(JSON.stringify(tc));

    })


  }

  async solvePuzzle(day: number, input: string): Promise<PuzzleResult[]> {
    await this.initialize();

    const parts = day === 25 ? [Part.Part1] : [Part.Part1, Part.Part2];
    return parts.map(part => {

      const response = solve_day(day, part, input);
      return response.error ? {part, error: response.error.toString()} : {part, result: response.result};
    });
  }
}
