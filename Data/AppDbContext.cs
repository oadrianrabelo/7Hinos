using Microsoft.EntityFrameworkCore;
using SevenHinos.Models;

namespace SevenHinos.Data;

public class AppDbContext(DbContextOptions<AppDbContext> options) : DbContext(options)
{
    public DbSet<Song> Songs => Set<Song>();
    public DbSet<SongSlide> SongSlides => Set<SongSlide>();
    public DbSet<FileAsset> FileAssets => Set<FileAsset>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<FileAsset>(e =>
        {
            e.HasKey(f => f.Id);
            e.Property(f => f.Id).ValueGeneratedOnAdd();
            e.Property(f => f.RelativePath).IsRequired();
            e.HasIndex(f => f.RelativePath).IsUnique();
        });

        modelBuilder.Entity<Song>(e =>
        {
            e.HasKey(s => s.Id);
            e.Property(s => s.Id).ValueGeneratedOnAdd();
            e.Property(s => s.Title).IsRequired();
            e.Property(s => s.Album).HasDefaultValue(string.Empty);
            e.Property(s => s.Lyrics).HasDefaultValue(string.Empty);
            e.HasMany(s => s.Slides)
             .WithOne(sl => sl.Song)
             .HasForeignKey(sl => sl.SongId)
             .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<SongSlide>(e =>
        {
            e.HasKey(sl => sl.Id);
            e.Property(sl => sl.Id).ValueGeneratedOnAdd();
            e.HasIndex(sl => new { sl.SongId, sl.Order }).IsUnique();
        });
    }
}
