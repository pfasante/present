clear
reset

bin_width = 1;

bin_number(x) = floor(x/bin_width)
rounded(x) = bin_width * ( bin_number(x) + 0.5 )

set terminal png size 1920,1080
#set xrange [-0.002:0.002]
#set yrange [0:20]
set key off
set border 3

# Add a vertical dotted line at x=0 to show centre (mean) of distribution.
set yzeroaxis

# Each bar is half the (visual) width of its x-range.
#set boxwidth 0.5 absolute
set style fill solid 1.0 noborder

# get list of files in data
list = system('ls data')

do for [file in list] {
    set output sprintf('plots/%s.png', file)
    plot "data/".file using 1:2 smooth frequency w boxes
}

