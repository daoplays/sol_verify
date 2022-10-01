use Archive::Extract;
 
### build an Archive::Extract object ###
my $ae = Archive::Extract->new( archive => $ARGV[0] );
 
### extract to test_repo ###
my $ok = $ae->extract( to => 'test_repo' ) or die $ae->error;
