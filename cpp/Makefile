all: clean build run plot view

build:
	clang++ -std=c++11 -O3 -pthread -o present present.cpp

run:
	./present

plot:
	gnuplot present.gp

view:
	pqiv plots/*.png

clean:
	rm -rf data/*
	rm -rf plots/*

phony: clean view
