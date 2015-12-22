override CFLAGS := -pedantic -pedantic-errors -Wall -std=c99 -O3 -flto $(CFLAGS)
override CXXFLAGS := -pedantic -pedantic-errors -Wall -std=c++11 -O3 -flto $(CXXFLAGS)
override LDFLAGS := -O3 -flto -lstdc++ $(LDFLAGS)


all: build build/present_std build/present_r2 build/checks

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


build/checks.o: src/checks.cpp src/present_bitslice.h
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

build/checks: build/checks.o
	$(CXX) $(LDFLAGS) $^ -o $@


clean:
	$(RM) -r build

dist: src/present.cpp src/present_bitslice.h src/cmdline.h src/keyschedule.h LICENSE README.md Makefile
	tar cvJpf present.tar.xz src LICENSE Makefile README.md

phony: clean
