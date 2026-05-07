import init, { CompiledRule } from 'jsonlogic-fast-wasm';

async function main() {
  await init();

  document.getElementById('eval-btn').addEventListener('click', () => {
    const ruleText = document.getElementById('rule').value;
    const contextText = document.getElementById('context').value;
    const resultDiv = document.getElementById('result');

    try {
      const start = performance.now();
      const rule = new CompiledRule(ruleText);
      const result = rule.evaluate(contextText);
      const end = performance.now();

      resultDiv.innerText = `${result} (evaluated in ${(end - start).toFixed(2)}ms)`;
      resultDiv.style.backgroundColor = '#d4edda';
    } catch (e) {
      resultDiv.innerText = `Error: ${e}`;
      resultDiv.style.backgroundColor = '#f8d7da';
    }
  });
}

main().catch(console.error);
