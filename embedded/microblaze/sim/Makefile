
CFLAGS = -Wall -g  -I/usr/include/libelf/
LDFLAGS = -lelf
OBJS = mb_core.o mb_mem.o mb_elf.o 


all: simulator 

mb_core.o: mb_core.c mb_autogen.py
	python mb_autogen.py	
	$(CC) $(CFLAGS) -c mb_core.c

simulator: main.o $(OBJS)
	$(CC) $(LDFLAGS) -o simulator main.o $(OBJS)

clean:
	rm -f *.o mb_autogen.c main *.tex *.elf

