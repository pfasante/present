present.o: present.cpp present.h present_bitsliced.h
	g++ -c -o present.o present.cpp -D_cplusplus -O3 -std=c++11 -pthread

present_bitsliced.o: present_bitsliced.c
	gcc -c -o present_bitsliced.o present_bitsliced.c -O3 -std=c11


build: present.o present_bitsliced.o
	g++ -o present present.o present_bitsliced.o -pthread

static: build
	g++ -Wl,--whole-archive -lpthread -Wl,--no-whole-archive -static -o present present.o present_bitsliced.o


test: test.cpp present.h present_bitsliced.h present_bitsliced.o
	g++ -D_cplusplus -o test test.o present_bitsliced.o


clean:
	rm -rf test
	rm -rf *.o

phony: clean view


run: build
	./present

plot:
	gnuplot present.gp

view:
	pqiv plots/*.png

