
#include <cmath>
#include <fstream>
#include <iostream>
#include <map>

#include "present.h"

using namespace std;

void print_estimated_mean_var(string name) {
    double mean = 0.0;
    double var = 2.0;
    var = pow(var, -4.0 * NUM_ROUNDS) * NUM_TRAILS[NUM_ROUNDS-1];

    ofstream output(name);
    if (!output.is_open()) {
        cout << "fatal error: could not open " << name << endl;
        return;
    }

    output << "mean " << mean << endl;
    output << "var  " << var << endl;
    output.close();
}

double weight(map<double, int> const& data) {
    double weight_sum = 0;
    for (auto const& x : data) {
        weight_sum += x.second;
    }
    return weight_sum;
}

double weighted_mean(map<double, int> const& data) {
    double mean = 0.0;
    double weight_sum = weight(data);
    for (auto const& x : data) {
        mean += x.first * x.second;
    }
    return mean / weight_sum;
}

double weighted_var(map<double, int> const& data) {
    double var = 0.0;
    double mean = weighted_mean(data);
    double weight_sum = weight(data);
    for (auto const& x : data) {
        var += pow(x.first - x.second * mean, 2.0);
        weight_sum += x.second;
    }
    return var / weight_sum;
}

void print_mean_var(p_corr_histo_t p_histo, string name) {
    double mean = 0.0;
    for (auto const& histo : (*p_histo)) {
        mean += weighted_mean(histo);
    }
    mean /= p_histo->size();

    double var = 0.0;
    for (auto const& histo : (*p_histo)) {
        var += weighted_var(histo);
    }
    var /= p_histo->size();

    ofstream output(name);
    if (!output.is_open()) {
        cout << "fatal error: could not open " << name << endl;
        return;
    }

    output << "mean " << mean << endl;
    output << "var  " << var << endl;
    output.close();
}

