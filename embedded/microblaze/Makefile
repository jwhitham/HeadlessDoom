################################################################
#
# $Id:$
#
# $Log:$
#
CC=  mb-gcc  # gcc or g++

CFLAGS=-g -O2 -DHEADLESS
LDFLAGS=-Wl,"--defsym=_HEAP_SIZE=0x600000" -Wl,"--defsym=_STACK_SIZE=0x10000"
LIBS=
OBJCOPY_BIN=mb-objcopy -I binary -O elf32-microblaze -B microblaze

# subdirectory for objects
O=obj

# subdirectory for Doom source files
HD=../../headless_doom

# not too sophisticated dependency
OBJS=				\
		$(O)/doomdef.o		\
		$(O)/doomstat.o		\
		$(O)/dstrings.o		\
		$(O)/i_system.o		\
		$(O)/i_sound.o		\
		$(O)/i_video.o		\
		$(O)/i_net.o			\
		$(O)/tables.o			\
		$(O)/f_finale.o		\
		$(O)/f_wipe.o 		\
		$(O)/d_main.o			\
		$(O)/d_net.o			\
		$(O)/d_items.o		\
		$(O)/g_game.o			\
		$(O)/m_menu.o			\
		$(O)/m_misc.o			\
		$(O)/m_argv.o  		\
		$(O)/m_bbox.o			\
		$(O)/m_fixed.o		\
		$(O)/m_swap.o			\
		$(O)/m_cheat.o		\
		$(O)/m_random.o		\
		$(O)/am_map.o			\
		$(O)/p_ceilng.o		\
		$(O)/p_doors.o		\
		$(O)/p_enemy.o		\
		$(O)/p_floor.o		\
		$(O)/p_inter.o		\
		$(O)/p_lights.o		\
		$(O)/p_map.o			\
		$(O)/p_maputl.o		\
		$(O)/p_plats.o		\
		$(O)/p_pspr.o			\
		$(O)/p_setup.o		\
		$(O)/p_sight.o		\
		$(O)/p_spec.o			\
		$(O)/p_switch.o		\
		$(O)/p_mobj.o			\
		$(O)/p_telept.o		\
		$(O)/p_tick.o			\
		$(O)/p_saveg.o		\
		$(O)/p_user.o			\
		$(O)/r_bsp.o			\
		$(O)/r_data.o			\
		$(O)/r_draw.o			\
		$(O)/r_main.o			\
		$(O)/r_plane.o		\
		$(O)/r_segs.o			\
		$(O)/r_sky.o			\
		$(O)/r_things.o		\
		$(O)/w_wad.o			\
		$(O)/wi_stuff.o		\
		$(O)/v_video.o		\
		$(O)/st_lib.o			\
		$(O)/st_stuff.o		\
		$(O)/hu_stuff.o		\
		$(O)/hu_lib.o			\
		$(O)/s_sound.o		\
		$(O)/z_zone.o			\
		$(O)/info.o				\
		$(O)/sounds.o           \
		$(O)/crc.o              \
		$(O)/timing.o           \
		$(O)/headless.o

all: headless_doom.elf
 
clean:
	rm -rf $(O) *~ headless_doom.elf

file_doom_wad.o:
	mb-objcopy -I binary -O elf32-microblaze $(HD)/doom.wad file_doom_wad.o

FILES=$(O)/file_DDQ-EP1.o \
        $(O)/file_DDQ-EP2.o \
        $(O)/file_DDQ-EP3.o \
        $(O)/file_DDQ-EP4.o \
        $(O)/file_doom.wad.o

$(HD)/doom.wad:
	make -C $(HD) doom.wad

$(HD)/DDQ-EP1.LMP:
	make -C $(HD) DDQ-EP1.LMP

$(O)/file_DDQ-EP%.o: $(HD)/DDQ-EP%.LMP
	$(OBJCOPY_BIN) $< $@

$(O)/file_doom.wad.o: $(HD)/doom.wad
	$(OBJCOPY_BIN) $< $@

headless_doom.elf:	$(OBJS) \
            $(O)/i_main.o $(O)/shim_libc.o $(O)/shim_asm.o $(FILES)
	$(CC) $(CFLAGS) $(LDFLAGS) \
        $(FILES) $(OBJS) \
        $(O)/i_main.o $(O)/shim_libc.o $(O)/shim_asm.o \
	-o headless_doom.elf $(LIBS)

$(O)/shim_libc.o:	shim_libc.c
	@mkdir -p $(O)
	$(CC) $(CFLAGS) -c $< -o $@

$(O)/shim_asm.o:	shim_asm.S
	@mkdir -p $(O)
	$(CC) $(CFLAGS) -c $< -o $@

$(O)/%.o:	$(HD)/%.c
	@mkdir -p $(O)
	$(CC) $(CFLAGS) -c $< -o $@

#############################################################
#
#############################################################
