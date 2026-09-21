/*
 * Task: Add the shell code in this file. The shell should run `ls -al / | tr a-z A-Z` using UNIX pipes. Please refer to the course webpage for more information
 */


#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>
#include <sys/types.h>

int producer()
{
        // Added exec call here
        execlp("ls", "ls", "-al", "/", NULL);
        return 0;
}

int consumer()
{
        // Added exec call here
        execlp("tr", "tr", "a-z", "A-Z", NULL);
        return 0;
}

int main()
{
        // Add a call to pipe here
        // define file descriptors
        int fds[2]; // 0 is read 1 is write
        pipe(fds); // creates the pipe

        // Fork off the consumer process here. Make sure to handle the file descriptors appropriately
        
        pid_t consumer_pid = fork();
        if (consumer_pid == 0) { // if copy process
                dup2(fds[0], STDIN_FILENO); // make duplicate stdin point to the read
                // pipe 0 now says read from the queue (stdin))
                close(fds[0]); // close the old read 
                close(fds[1]); // close the old write
                consumer(); // calls the tr process
        }

        // Fork off the producer process here. Make sure to handle the file descriptors appropriately
        pid_t producer_pid = fork();
        if (producer_pid == 0) {
                dup2(fds[1], STDOUT_FILENO); // make the stdout point to the write
                close(fds[0]);
                close(fds[1]);
                producer(); // calls the ls process
        }
        
        // Handle the file descriptors for the shell process, i.e. this process here
        close(fds[0]);
        close(fds[1]);
        // Add a call to waitpid for the consumer process here
        // Add a call to waitpid for the producer process here
        waitpid(consumer_pid, NULL, 0);
        waitpid(producer_pid, NULL, 0);

        return 0;
}
