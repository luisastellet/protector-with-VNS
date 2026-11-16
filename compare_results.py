import os
import pandas as pd

files = {
    "Greedy (Baseline)": "results/resumo_one-greedy-construction.csv",
    "GRASP (Baseline)": "results/resumo_grasp.csv",
    "VNS (Proposto)": "results/resumo_vns.csv",
}

data = []
for name, path in files.items():
  if os.path.exists(path):
    df = pd.read_csv(path)
    data.append({
        "Método": name,
        "Melhor Score": df["melhor_score"].max(),
        "Score Médio": df["melhor_score"].mean(),
        "Desvio Padrão": df["melhor_score"].std(),
        "Tempo Médio (s)": df["tempo_segundo"].mean(),
    })
  else:
    print(f"[Aviso] Arquivo não encontrado: {path}")

if data:
  res_df = pd.DataFrame(data)
  print("\n" + "=" * 80)
  print(res_df.to_markdown(index=False))
  print("=" * 80 + "\n")
else:
  print("Nenhum arquivo de resultado encontrado para comparar.")