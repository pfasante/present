override CFLAGS := -pedantic -pedantic-errors -Wall -std=c99 -O3 $(CFLAGS)
override CXXFLAGS := -pedantic -pedantic-errors -Wall -std=c++11 -O3 $(CXXFLAGS)
override LDFLAGS := -lstdc++ $(LDFLAGS)

all: build build/present

build:
	mkdir -p build

build/%.o: src/%.c
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

build/%.o: src/%.cpp
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

build/main.o: src/main.cpp src/present_bitslice.h src/cmdline.h src/keyschedule.h
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -pthread -c $< -o $@

build/present: build/main.o build/present_bitslice.o build/cmdline.o
	$(CXX) $(LDFLAGS) -pthread $^ -o $@

clean:
	$(RM) -r build

phony: clean
