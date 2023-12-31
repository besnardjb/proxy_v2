/*
 * Check decoding of accept syscall.
 *
 * Copyright (c) 2016 Dmitry V. Levin <ldv@strace.io>
 * Copyright (c) 2016-2023 The strace developers.
 * All rights reserved.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "tests.h"
#include "scno.h"
#include <unistd.h>

#ifndef TEST_SYSCALL_NAME

# if defined __NR_accept || defined __NR_socketcall

#  define TEST_SYSCALL_NAME do_accept
#  define TEST_SYSCALL_STR "accept"

static int
do_accept(int sockfd, void *addr, void *addrlen)
{
#  ifdef __NR_accept
	return syscall(__NR_accept, sockfd, addr, addrlen);
#  else /* __NR_socketcall */
	const long args[] = { sockfd, (long) addr, (long) addrlen };
	return syscall(__NR_socketcall, 5, args);
#  endif
}

# endif /* __NR_accept || __NR_socketcall */

#endif /* !TEST_SYSCALL_NAME */

#ifdef TEST_SYSCALL_NAME

# define TEST_SYSCALL_PREPARE connect_un()
static void connect_un(void);
# include "sockname.c"

static void
connect_un(void)
{
	int cfd = socket(AF_UNIX, SOCK_STREAM, 0);
	if (cfd < 0)
		perror_msg_and_skip("socket");

	struct sockaddr_un un = {
		.sun_family = AF_UNIX,
		.sun_path = TEST_SOCKET ".connect"
	};

	(void) unlink(un.sun_path);
	if (bind(cfd, (const void *) &un, sizeof(un)))
		perror_msg_and_skip("bind");
	(void) unlink(un.sun_path);

	un.sun_path[sizeof(TEST_SOCKET) - 1] = '\0';
	if (connect(cfd, (const void *) &un, sizeof(un)))
		perror_msg_and_skip("connect");
}

int
main(void)
{
	int lfd = socket(AF_UNIX, SOCK_STREAM, 0);
	if (lfd < 0)
		perror_msg_and_skip("socket");

	(void) unlink(TEST_SOCKET);

	const struct sockaddr_un un = {
		.sun_family = AF_UNIX,
		.sun_path = TEST_SOCKET
	};

	if (bind(lfd, (const void *) &un, sizeof(un)))
		perror_msg_and_skip("bind");
	if (listen(lfd, 16))
		perror_msg_and_skip("listen");

	test_sockname_syscall(lfd);

	(void) unlink(TEST_SOCKET);

	puts("+++ exited with 0 +++");
	return 0;
}

#else

SKIP_MAIN_UNDEFINED("__NR_accept || __NR_socketcall")

#endif
