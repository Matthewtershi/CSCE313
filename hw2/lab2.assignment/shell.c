/*
 * Task: Add the shell code in this file. The shell should run `ls -al / | tr a-z A-Z` using UNIX pipes. Please refer to the course webpage for more information
 */


#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>

int producer()
{
        // Added exec call here
        return 0;
}

int consumer()
{
        // Added exec call here
        return 0;
}

int main()
{
        // Add a call to pipe here
        // Fork off the consumer process here. Make sure to handle the file descriptors appropriately
        // Fork off the producer process here. Make sure to handle the file descriptors appropriately
        // Handle the file descriptors for the shell process, i.e. this process here
        // Add a call to waitpid for the consumer process here
        // Add a call to waitpid for the producer process here

        return 0;
}
