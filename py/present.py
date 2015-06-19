#!/usr/bin/python2

from functools import reduce
from math import ceil, copysign, log, pow
from operator import mul
from sys import argv

import random

import numpy
from numpy import linalg as LA

import pydot


def sbox(i):
    return [0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
            0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2][i]


def inv(sbox, x):
    return [sbox.index(i) for i in range(len(sbox))][x]


def permute(x):
    y = 0
    for j in range(16):
        for i in range(4):
            oldIdx = j*4+i
            newIdx = i*16+j
            bit = x >> oldIdx & 0x1
            y |= bit << newIdx
    return y


def permuteIdx(x):
    """
    computes the new index of input index
    """
    i = x % 4
    j = x // 4
    return i * 16 + j


def keyschedule(key):
    """
    returns a list of the PRESENT roundkeys
    """
    keys = []
    for i in range(1, 32):
        keys.append(key >> 16)
        rotatedKey = ((key & 0x7ffff) << 61) | (key >> 19)
        leftmostnibble = (rotatedKey & (0xf << 76)) >> 76
        substitutedKey = rotatedKey ^ (leftmostnibble << 76)
        substitutedKey |= (sbox(leftmostnibble) << 76)
        key = substitutedKey ^ ((i % 0x1f) << 15)
    return keys


def dotproductF2(a, b):
    """
    computes the dotproduct in F_2
    """
    n = int(max([ceil(log(a + 1, 2)), ceil(log(b + 1, 2))]))
    x = 0
    for i in range(n):
        x ^= ((a >> i) & 1) * ((b >> i) & 1)
    return x


def walshTransform(f, domain, beta):
    result = [(-1)**dotproductF2(beta, f(i)) for i in range(domain)]
    step = 1
    while (step < domain):
        left = 0
        numOfBlocks = int(round(domain / (step * 2)))
        for i in range(numOfBlocks):
            right = left + step
            for j in range(step):
                a, b = result[left], result[right]
                result[left], result[right] = a + b, a - b
                left, right = left + 1, right + 1
            left = right
        step *= 2
    return result


def LAT(f, domain, image):
    """
    f is the function for which we compute the linear approximation table
    with given domain and image
    """
    table = []
    for i in range(image):
        table.append(walshTransform(f, domain, i))
    return table


def biasedLinApproxOneBit():
    """
    returns the biased (!= 0) linear approximations with one bit input/output
    masks of the present sbox
    """
    table = LAT(sbox, 16, 16)
    transposed = list(map(list, zip(*table)))
    oneBitInOutMasks = [(1 << a, 1 << b) for a in range(4) for b in range(4)]
    result = []
    for (i, j) in oneBitInOutMasks:
        if transposed[i][j] != 0:
            result.append((i, j, transposed[i][j]))
    return result


def followCharacteristic(a):
    """
    returns a list of all possible values for following each of the
    one bit input output masks

    substitutes the input with all possible output masks,
    permutes the result
    and save it for the output
    """
    result = []
    for m in biasedLinApproxOneBit():
        for i in range(16):
            if ((m[0] << (4 * i)) == (a[0] & (0xf << (4 * i)))):
                t = (a[0] & (~(0xf << (4 * i)))) | (m[1] << (4 * i))
                result += [(permute(t), a[1] + [m])]
    # print(a, result)
    return result


def linCharacteristics(alpha, beta, rounds):
    trails = [(alpha, [])]
    for i in range(rounds):
        newtrails = []
        for t in trails:
            newtrails += followCharacteristic(t)
        trails = newtrails[:]
    # print([(hex(t[0]), t[1]) for t in trails], len(trails))
    trails = [t for t in trails if t[0] == beta]
    return trails


def numberOfTrails(alpha, beta, rounds, keys=None):
    """
    the input/output masks should denote the bit, which is set in the state,
    i.e. alpha = 1, beta = 4 denotes 0x000...02 -> 0x000...08
    thus the second inbit of the first sbox -> 4 outbit of the first sbox
    """
    masks = biasedLinApproxOneBit()
    trails = [(alpha, alpha, 1, 1)]
    # state tuple consists of (in mask, out mask, sign of prob, key dependency)
    if keys is None:
        for i in range(rounds):
            newtrails = []
            for (a, b, p, _) in trails:
                # update trail here
                for (_a, _b, _p) in masks:
                    activeSbox = b // 4
                    if (_a == 2**(b % 4)):
                        c = permuteIdx(int(log(_b, 2)) + activeSbox * 4)
                        newtrails.append((a, c, int(copysign(1, p * _p)), 1))
            trails = newtrails
    else:
        for i in range(rounds):
            newtrails = []
            for (a, b, p, d) in trails:
                # update trail here
                for (_a, _b, _p) in masks:
                    activeSbox = b // 4
                    if (_a == 2**(b % 4)):
                        c = permuteIdx(int(log(_b, 2)) + activeSbox * 4)
                        newsign = int(copysign(1, p * _p))
                        newkeydep = d * (-1)**(1 - int((keys[i] & b) > 0))
                        newtrails.append((a, c, newsign, newkeydep))
            trails = newtrails
    filteredTrails = [t for t in trails if t[1] == beta]
    trailsWithProb = []
    for (a, b, sign, d) in filteredTrails:
        prob = sign * pow(0.125, rounds)
        trailsWithProb.append((a, b, prob * d))
    return trailsWithProb


def countTrails(alpha, beta, rounds):
    """
    counts the number of trails in a slightly more sophisticated way
    than numberOfTrails
    """
    masks = biasedLinApproxOneBit()
    adjmatr = [[0 for _ in range(64)] for _ in range(64)]
    for i in range(64):
        # update trail here
        for (a, b, _) in masks:
            activeSbox = i // 4
            if (a == 2**(i % 4)):
                c = permuteIdx(int(log(b, 2)) + activeSbox * 4)
                adjmatr[c][i] = 1

    empty = [[0]*64]
    g1matr = empty*64
    for i in [16, 32, 48]:
        g1matr[i] = adjmatr[i]

    g2matr = empty*64
    for i in [5, 6, 7, 9,
              10, 11, 13, 14,
              15, 17, 18, 19,
              20, 24, 28, 33,
              34, 35, 36, 40,
              44, 49, 50, 51,
              52, 56, 60]:
        g2matr[i] = adjmatr[i]

    g3matr = empty*64
    for i in [21, 22, 23,
              25, 26, 27,
              29, 30, 31,
              37, 38, 39,
              41, 42, 43,
              45, 46, 47,
              53, 54, 55,
              57, 58, 59,
              61, 62, 63]:
        g3matr[i] = adjmatr[i]

    # g1 = graph_from_adjacency_matrix(g1matr, u'', True)
    # g2 = graph_from_adjacency_matrix(g2matr, u'', True)
    # g3 = graph_from_adjacency_matrix(g3matr, u'', True)
    # with open("g1.dot", "w") as f:
    #     f.write(g1.to_string())
    # with open("g2.dot", "w") as f:
    #     f.write(g2.to_string())
    # with open("g3.dot", "w") as f:
    #     f.write(g3.to_string())

    numpy_retmatr = LA.matrix_power(numpy.matrix(adjmatr), rounds)
    # print(rounds, log(adjmatr.max(), 2), adjmatr.max())
    # adjmatr = numpy_retmatr.tolist()
    # numpy.set_printoptions(linewidth=numpy.nan, threshold=numpy.nan)
    # # print(numpy_retmatr)
    # g = graph_from_adjacency_matrix(adjmatr, u'', True)
    # with open(str(rounds)+"rounds.dot", "w") as f:
    #     f.write(g.to_string())

    return numpy_retmatr


def count_trails_identical_keys(key, rounds):
    masks = biasedLinApproxOneBit()
    adjmatr = [[0 for _ in range(64)] for _ in range(64)]
    for i in range(64):
        # update trail here
        for (a, b, p) in masks:
            activeSbox = i // 4
            if (a == 2**(i % 4)):
                c = permuteIdx(int(log(b, 2)) + activeSbox * 4)
                adjmatr[c][i] = p // 4

    keymatr = [[0 for _ in range(64)] for _ in range(64)]
    for i in range(64):
        keymatr[i][i] = (-1)**((key >> i) & 1)

    numpy_adjmatr = numpy.matrix(adjmatr)
    numpy_keymatr = numpy.matrix(keymatr)
    numpy_retmatr = LA.matrix_power(numpy_adjmatr * numpy_keymatr, rounds)

    numpy.set_printoptions(linewidth=numpy.nan, threshold=numpy.nan)

    # print(numpy_retmatr.max())
    # print(rounds, log(numpy_retmatr.max(), 2), numpy_retmatr.max())

    return numpy_retmatr


def graph_from_adjacency_matrix(matrix, node_prefix=u'', directed=False):
    if directed:
        graph = pydot.Graph(graph_name='G', graph_type='digraph', strict=False,
                            suppress_disconnected=True)
    else:
        graph = pydot.Graph(graph_name='G', graph_type='graph', strict=False,
                            suppress_disconnected=True)
    for i in range(len(matrix)):
        for j in range(len(matrix[i])):
            if matrix[i][j]:
                graph.add_edge(pydot.Edge(str(i), str(j)))
    return graph

###############################################################################


def print_table(L):
    flat = []
    for i in L:
        for j in i:
            flat.append(j)

    m = 16
    line = ' '

    for i in range(m):
        line += ' %2x' % i

    for i, l in enumerate(flat):
        if not i % m:
            line += '\n%1x' % (i // m)
        if l == 0:
            line += ' ' * 3
        else:
            line += ' %2d' % l

    line += '\n'
    print(line)


def naiveLAT(f, domain, image):
    table = [[0 for i in range(image)] for j in range(domain)]
    for i in range(domain):
        for j in range(image):
            for x in range(domain):
                table[j][i] += 1 - dotproductF2(i, x) ^ dotproductF2(j, f(x))
            table[j][i] -= domain // 2
    return table


# def sboxLin(i):
#     return [0xf, 0xe, 0xb, 0xc, 0x6, 0xd, 0x7, 0x8,
#             0x0, 0x3, 0x9, 0xa, 0x4, 0x2, 0x1, 0x5][i]


if __name__ == '__main__':
    rounds = int(argv[1])
    number_keys = int(argv[2])

    numpy_indpmatr = countTrails(1, 1, rounds)
    max_indpcount = numpy_indpmatr.max()
    print("indp", rounds, log(max_indpcount, 2), max_indpcount)

    numpy_retmatr = numpy.matrix([[0 for _ in range(64)] for _ in range(64)])
    for _ in range(number_keys):
        key = random.randint(0, (1 << 64)-1)
        numpy_temp = count_trails_identical_keys(key, rounds)
        for i in range(64):
            for j in range(64):
                numpy_retmatr[i, j] += numpy_temp[i, j]**2

    max_count = numpy_retmatr.max()
    print("key", rounds, max_count / float(number_keys))
    print("key", rounds, max_count)
    # print(numpy_retmatr / float(number_keys))
    # print(rounds, log(max_count // number_keys, 2), max_count // number_keys)
    # print(rounds, log(max_count, 2), max_count)

    for i in range(64):
        for j in range(64):
            if numpy_indpmatr.tolist()[i][j] == max_indpcount:
                print(i, j, "indp", max_indpcount,
                      "key", numpy_retmatr.tolist()[i][j] / float(number_keys))
            if numpy_retmatr.tolist()[i][j] == max_count:
                print(i, j, "indp", numpy_indpmatr.tolist()[i][j],
                      "key", max_count / float(number_keys))

#    trails = countTrails(21, 21, int(argv[1]))
#    print("1. Compute the linear approximation table for the PRESENT S-box")
#    table = LAT(sbox, 16, 16)
#    transposed = map(list, zip(*table))
#    print_table(transposed)
#
#    print("2. Find all biased linear approximation with a one bit input and "
#          "output mask.")
#    print(biasedLinApproxOneBit())
#    print("")
#
#    print("3. Find a linear characteristic for some rounds with only one "
#          "active Sbox per round")
#    trails = linCharacteristics(0x200000, 0x200000, 4)
#    print([(hex(t[0]), t[1]) for t in trails][0],
#          "\nout of", len(trails), "trails")
#    print("")
#
#    print("4. Compute its bias.")
#    biases = map(lambda x: reduce(mul, x),
#                 [[t[2] for t in ts[1]] for ts in trails])
#    print("|bias| = sign * pow(2/16, rounds)\nbias =",
#          copysign(1, list(biases)[0]) * pow(0.125, 4)
#          )
#    print("")
#
#    print("5. Why do all these characteristic have the same absolute bias?")
#    print("All one-bit input and output masks have the same absolute bias of "
#          "1/8, thus every linear characteristic from these masks has the "
#          "same bias.")
#    print("")
#
#    print("6. For any given one-bit input and output mask: Find the total "
#          "number of linear characteristics in the linear hull over r rounds "
#          "with only one active S-box per round.")
#    if len(argv) > 1:
#        rounds = int(argv[1])
#    else:
#        rounds = 13
# #    trails = numberOfTrails(21, 21, rounds)
#    trails = countTrails(21, 21, rounds)
#    print("rounds", rounds,
#          "number of trails N_T", trails,
#          "log N_T", round(log(trails, 2), 2))
#    print("")
#
#    print("7. Execute some experiments to see how the bias is distributed "
#          "over the keys.")
#    if len(argv) > 1:
#        rounds = int(argv[1])
#    else:
#        rounds = 10
#    key = 0x0
#    trails = numberOfTrails(21, 21, rounds, keyschedule(key))
#    correlation = 0
#    for (_, _, p) in trails:
#        correlation += p
#    print("rounds", rounds,
#          "key", hex(key),
#          "correlation for linear hull", correlation
#          )
#    print("")
