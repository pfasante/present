override CFLAGS := -pedantic -pedantic-errors -Wall -std=c99 -O3 $(CFLAGS)
override CXXFLAGS := -pedantic -pedantic-errors -Wall -std=c++11 -O3 $(CXXFLAGS)
override LDFLAGS := -lstdc++ $(LDFLAGS)

all: build build/present_std build/present_r2

build:
	mkdir -p build

build/%.o: src/%.c
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

build/%.o: src/%.cpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

build/present_r2.o: src/present.cpp src/present_bitslice.h src/cmdline.h src/keyschedule.h
	$(CXX) $(CPPFLAGS) -DUSE_SBOX=Sbox_R2 $(CXXFLAGS) -pthread -c $< -o $@

build/present_std.o: src/present.cpp src/present_bitslice.h src/cmdline.h src/keyschedule.h
	$(CXX) $(CPPFLAGS) -DUSE_SBOX=Sbox_Present $(CXXFLAGS) -pthread -c $< -o $@

build/present_r2: build/present_r2.o build/cmdline.o
	$(CXX) $(LDFLAGS) -pthread $^ -o $@

build/present_std: build/present_std.o build/cmdline.o
	$(CXX) $(LDFLAGS) -pthread $^ -o $@

clean:
	$(RM) -r build

dist: src/present.cpp src/present_bitslice.h src/cmdline.h src/keyschedule.h LICENSE README.md Makefile
	tar cvJpf present.tar.xz src LICENSE Makefile README.md

phony: clean
