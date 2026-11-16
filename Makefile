.PHONY: all setup build run-greedy run-grasp run-vns run-all compare clean

all: setup build

setup:
	@echo "==> Configurando dependências Python e ambiente..."
	python3 -m pip install --upgrade pip
	@if [ -f requirements.txt ]; then pip install -r requirements.txt; fi
	@echo "✔ Setup concluído!"

build:
	@echo "==> Compilando o núcleo de otimização em Rust (Modo Release)..."
	cargo build --release
	@echo "✔ Compilação concluída!"

run-greedy:
	@echo "==> Executando Heurística Gulosa (Greedy)..."
	cargo run --release -- instances/sacoma-3745/graph --metaheuristic one-greedy-construction --executions 10 --max-section-size 0.3719 --iterations 1

run-grasp:
	@echo "==> Executando GRASP puro (Parâmetros originais: 100 iterações, 10 execuções)..."
	cargo run --release -- instances/sacoma-3745/graph --metaheuristic grasp --iterations 100 --executions 10 --alpha 0.1436 --max-section-size 0.3719

run-vns:
	@echo "==> Executando VNS / VND híbrido com os parâmetros originais do artigo..."
	cargo run --release -- instances/sacoma-3745/graph --metaheuristic vns --iterations 100 --executions 10 --alpha 0.1436 --max-section-size 0.3719

run-all: run-greedy run-grasp run-vns
	@echo "✔ Todos os experimentos foram concluídos!"

compare:
	@echo "==> Analisando e comparando os resultados..."
	python3 compare_results.py

clean:
	@echo "==> Limpando artefatos..."
	cargo clean
	rm -rf __pycache__ .pytest_cache *.pyc *.csv results/
	@echo "✔ Limpeza concluída!"