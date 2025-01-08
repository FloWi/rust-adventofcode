// main.ts
import {AocSolver} from './solver';

class AocUI {
  private solver: AocSolver;
  private daySelect: HTMLSelectElement;
  private inputArea: HTMLTextAreaElement;
  private solutionDiv: HTMLElement;
  private part1Pre: HTMLElement;
  private part2Pre: HTMLElement;

  constructor() {
    this.solver = new AocSolver();
    this.daySelect = document.getElementById('day') as HTMLSelectElement;
    this.inputArea = document.getElementById('input') as HTMLTextAreaElement;
    this.solutionDiv = document.getElementById('solution') as HTMLElement;
    this.part1Pre = document.getElementById('part1') as HTMLElement;
    this.part2Pre = document.getElementById('part2') as HTMLElement;

    // Bind solve method to window for the onclick handler
    (window as any).solve = this.solve.bind(this);
  }

  private async solve(): Promise<void> {
    try {
      const day = parseInt(this.daySelect.value);
      const input = this.inputArea.value;

      const results = await this.solver.solvePuzzle(day, input);

      this.solutionDiv.style.display = 'block';


      // Handle Part 1
      const part1 = results.find(r => r.part === 1);
      console.log(part1);

      this.part1Pre.textContent = part1?.error || part1?.result || 'Not computed';

      // Handle Part 2
      const part2 = results.find(r => r.part === 2);
      console.log(part2);
      this.part2Pre.textContent = part2?.error || part2?.result || 'Not computed';

    } catch (e) {
      const errorDiv = document.createElement('div');
      errorDiv.className = 'error';
      errorDiv.textContent = e instanceof Error ? e.message : String(e);
      this.solutionDiv.insertAdjacentElement('beforebegin', errorDiv);
    }
  }

  async initialize(): Promise<void> {
    await this.solver.initialize();
    await this.solver.printTestcases();
  }
}

// Initialize the application
const app = new AocUI();
app.initialize();
