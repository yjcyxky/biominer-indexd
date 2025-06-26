from setuptools import setup, find_packages

setup(
    name='biominer-idxd-convertor',
    version='0.1.0',
    description='Data convertor for BioMiner Indexd.',
    author='Jingcheng Yang (yjcyxky@163.com)',
    packages=find_packages(),
    install_requires=[
        'click',
        'pandas',
        'numpy',
        'pyarrow',
    ],
    entry_points={
        'console_scripts': [
            'biominer-idxd=biominer_idxd_convertor.cli:cli',
        ],
    },
    python_requires='>=3.7',
) 