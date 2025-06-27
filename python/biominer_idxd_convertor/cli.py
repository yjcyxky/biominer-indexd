import click
from pathlib import Path
from .cbioportal2dataset import build_mappings, convert_cbioportal_study
from . import omics

@click.group()
def cli():
    """biominer-idxd command line tool"""
    pass

@cli.command()
@click.argument('study_dir', type=click.Path(exists=True, file_okay=False))
@click.argument('output_dir', type=click.Path())
@click.option('--organization', type=str, default='Unassigned')
@click.option('--version', type=str, default='v0.0.1')
@click.option('--skip', is_flag=True, default=False, help='Skip already converted studies')
def convert(study_dir, output_dir, organization, version, skip):
    """Convert cBioPortal dataset to standard format"""
    build_mappings()
    out_dir = convert_cbioportal_study(study_dir, output_dir, organization, version, skip)
    omics.convert_all_omics(Path(study_dir), Path(out_dir) / 'datafiles', skip)
    click.echo(f'✅ Conversion completed, output directory: {out_dir}')

@cli.command()
@click.argument('study_dir', type=click.Path(exists=True, file_okay=False))
@click.argument('output_dir', type=click.Path())
@click.option('--organization', type=str, default='Unassigned')
@click.option('--version', type=str, default='v0.0.1')
@click.option('--skip', is_flag=True, default=False, help='Skip already converted studies')
def bconvert(study_dir, output_dir, organization, version, skip):
    """Backup bulk conversion command (currently same as convert)"""
    for sub_study_dir in sorted(Path(study_dir).iterdir()):
        if sub_study_dir.is_dir():
            output_study_dir = Path(output_dir) / sub_study_dir.name
            build_mappings()
            out_dir = convert_cbioportal_study(sub_study_dir, output_study_dir, organization, version, skip)
            omics.convert_all_omics(Path(sub_study_dir), Path(out_dir) / 'datafiles', skip)
            click.echo(f'✅ Conversion completed, output directory: {out_dir}')

if __name__ == '__main__':
    cli() 