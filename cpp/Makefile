all: clean build run plot view

build:
	gcc -c -o present_bitsliced.o present_bitsliced.c -O3 -std=c11
	g++ -c -o present.o present.cpp -D_cplusplus -O3 -std=c++11 -pthread
	g++ -o present present.o present_bitsliced.o -pthread

static: build
	g++ -Wl,--whole-archive -lpthread -Wl,--no-whole-archive -static -o present present.o present_bitsliced.o

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
