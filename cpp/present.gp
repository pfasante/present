clear
reset

bin_width = 1;

Gauss(x, mu, sigma) = 1./(sqrt(sigma*2*pi)) * exp(-(x-mu)**2 / (2*sigma))
bin_number(x) = floor(x/bin_width)
rounded(x) = bin_width * ( bin_number(x) + 0.5 )

set terminal png size 1920,1080
#set key off
set border 3

#set xrange [-0.06:0.06]
#set yrange [0:150]

# Add a vertical dotted line at x=0 to show centre (mean) of distribution.
set yzeroaxis

# Each bar is half the (visual) width of its x-range.
#set boxwidth 0.5 absolute
set style fill solid 1.0 noborder

stats 'data/data_est' using 2 name 'dataest'
var_est = dataest_max
mu_est = dataest_min

set output "plots/histo_indp.png"
set title "Histogram for Independent Round Keys"
stats 'data/data_indp' using 2 name 'dataindp'
var_indp = dataindp_max
mu_indp = dataindp_min
list_indp = system('ls data/histo_indp*')
plot for [file in list_indp] file using 1:2 notitle lc rgb "light-grey" smooth frequency w boxes, \
    Gauss(x, mu_indp, var_indp) title "Gauss measured" lc rgb '#ff0000', \
    Gauss(x, mu_est, var_est) title "Gauss estimated" lc rgb '#00ff00'

set output "plots/histo_id.png"
set title "Histogram for Identical Round Keys"
stats 'data/data_indp' using 2 name 'dataid'
var_id = dataid_max
mu_id = dataid_min
list_id = system('ls data/histo_id*')
plot for [file in list_id] file using 1:2 notitle lc rgb "light-grey" smooth frequency w boxes, \
    Gauss(x, mu_id, var_id) title "Gauss measured" lc rgb '#ff0000', \
    Gauss(x, mu_est, var_est) title "Gauss estimated" lc rgb '#00ff00'


#set output "plots/histo_no.png"
#set title "Histogram for Null Round Key"
#stats 'data/data_no' using 2 name 'datano'
#var_no = datano_max
#mu_no = datano_min
#list_no = system('ls data/histo_no*')
#plot for [file in list_no] file using 1:2 notitle lc rgb "light-grey" smooth frequency w boxes, \
#    Gauss(x, mu_no, var_no) title "Gauss measured" lc rgb '#ff0000', \
#    Gauss(x, mu_est, var_est) title "Gauss estimated" lc rgb '#00ff00'


#do for [file in list] {
#    set output sprintf('plots/%s.png', file)
#    plot "data/".file using 1:2 smooth frequency w boxes
#}

