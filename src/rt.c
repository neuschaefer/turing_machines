#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <locale.h>
#include <errno.h>
#include <ctype.h>
#include <wchar.h>

#include <sys/mman.h>

typedef uint32_t *(*tm_func_t)(uint32_t *);

#define TAPE_SIZE	(1u<<31)	/* 2 GiB */
#define STDIN_BUFSIZE	8192		/* 8 KiB */

#define PREFIX "libturingrt: "		/* a prefix for error messages */

extern void tm_run(tm_func_t , uint32_t *, uint32_t);
extern void tm_fail(const char *state, uint32_t symbol);

static void read_input(char *, size_t, wchar_t *, uint32_t *, uint32_t);
static void write_output(wchar_t *, uint32_t *, uint32_t);

extern void tm_run(tm_func_t fn, uint32_t *isyms, uint32_t num_isyms)
{
	// setup the memory
	uint8_t *mmap_res = mmap(NULL, TAPE_SIZE, PROT_READ|PROT_WRITE,
			MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);

	if (mmap_res == MAP_FAILED) {
		perror(PREFIX "Failed to acquire tape memory");
		exit(EXIT_FAILURE);
	}

	// TODO: Guard pages at both ends of the tape

	// The initial tape pointer
	uint8_t *tape_start = mmap_res + TAPE_SIZE/2;

	// Use the rightmost part of the left half of tape as a temporary
	// buffer for undecoded (hopefully UTF-8) input from stdin.
	uint8_t *stdin_buffer = tape_start - STDIN_BUFSIZE;

	// read stdin, into stdin_buffer, decode it on the fly
	read_input(stdin_buffer, STDIN_BUFSIZE, (wchar_t *)tape_start,
			isyms, num_isyms);

	// clear the buffer so it can safely serve as part of the tape again.
	memset(stdin_buffer, 0, STDIN_BUFSIZE);

	uint32_t *new_tp = fn((uint32_t *)tape_start);

	write_output(new_tp, isyms, num_isyms);
}

static bool is_isym(uint32_t which, uint32_t *isyms, uint32_t num_isyms)
{
	uint32_t i;
	for (i = 0; i < num_isyms; i++)
		if (which == isyms[i])
			return true;
	return false;
}

static void read_input(char *buf, size_t bufsize, wchar_t *tape,
		uint32_t *isyms, uint32_t num_isyms)
{
	// UTF-8 encoding, but no language specific settings.
	setlocale(LC_ALL, "C.UTF-8");

	size_t bytes;
	mbstate_t state;

	for (;;) {
		size_t offset = 0;
		wchar_t wc;

		errno = 0;
		bytes = fread(buf, 1, bufsize, stdin);
		if (bytes == 0)
			break;

		/* decode each character, store input characters on the tape */
		while (offset < bytes) {
			int res = mbrtowc(&wc, buf+offset, bufsize-offset,
					&state);

			if (res > 0) {
				offset += res;
				if (is_isym(wc, isyms, num_isyms))
					*(tape++) = wc;
			} else {
				/* skip a byte on errors and null bytes */
				offset += 1;
			}
		}
	}

	if (errno != 0) {
		perror(PREFIX "failed to read from standard input");
		exit(EXIT_FAILURE);
	}
}

static void write_output(wchar_t *tape, uint32_t *isyms, uint32_t num_isyms)
{
	wchar_t *p;
	for (p = tape; is_isym(*p, isyms, num_isyms); p++) {
		printf("Symbol U+%04X in output stream\n", *p);
		// TODO!
	}
}

#define P(...) fprintf(stderr, PREFIX __VA_ARGS__)
extern void tm_fail(const char *state, uint32_t symbol)
{
	if (symbol == 0) {
		P("No transition from %s on symbol blank (U+0000)\n",
				state, symbol);
	} else if (isprint(symbol)) {
		P("No transition from %s on symbol '%c' (U+%04X)\n",
				state, symbol, symbol);
	} else {
		// TODO: encode the symbol as UTF-8 for output
		P("No transition from %s on symbol U+%04X\n", state, symbol);
	}

	exit(EXIT_FAILURE);
}
#undef P
