PYTHON     := .venv/bin/python
COMPARISON := comparison.csv
PLOT       := gains.png
OUTLIERS   := outliers.txt

.PHONY: all bench plot clean

all: bench plot

bench: $(COMPARISON)

$(COMPARISON):
	$(PYTHON) bench-compare.py -o $(COMPARISON)

plot: $(PLOT)

$(PLOT): $(COMPARISON)
	$(PYTHON) plot-gains.py -i $(COMPARISON) -o $(PLOT) --outliers $(OUTLIERS)

clean:
	rm -f $(COMPARISON) bench_baseline.csv bench_u256.csv $(PLOT) $(OUTLIERS)
